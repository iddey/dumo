//! Display module with a resolution of 170x320 and an ST7789V3 controller
//! Configured for an 8-bit parallel interface to an ESP32-S3

#![no_std]
#![no_main]

use core::array;
use core::iter;

use dumo::DumoBackend;
use dumo::fonts::*;
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_sync::zerocopy_channel::{Channel, Sender};
use embassy_time::{Delay, Duration, with_timeout};
use embedded_graphics::pixelcolor::Rgb888;
use esp_hal::Config;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::system::Stack;
use esp_hal::timer::timg;
use heapless::Vec;
use mipidsi::Builder;
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::models::ST7789;
use mipidsi::options::ColorInversion;
use panic_halt as _;
use ratatui::layout::Constraint::Length;
use ratatui::layout::Offset;
use ratatui::prelude::*;
use ratatui::widgets::{Block, RatatuiLogo};
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

const FG_COLOR: Rgb888 = Rgb888::new(246, 214, 187);
const BG_COLOR: Rgb888 = Rgb888::new(20, 20, 50);

const fn get_color(row_index: usize, column_index: usize) -> Color {
    const RED_GRADIENT: [u8; 6] = [41, 43, 50, 68, 104, 156];
    const GREEN_GRADIENT: [u8; 6] = [24, 30, 41, 65, 105, 168];
    const BLUE_GRADIENT: [u8; 6] = [55, 57, 62, 78, 113, 166];
    const AMBIENT_GRADIENT: [u8; 6] = [17, 18, 20, 25, 40, 60];

    let index = if row_index < 6 { 5 - row_index } else { 0 };
    let ambient = AMBIENT_GRADIENT[index];
    let red = RED_GRADIENT[index];
    let green = GREEN_GRADIENT[index];
    let blue = BLUE_GRADIENT[index];
    let blue_sat = ambient.saturating_mul(6 - index as u8);
    let blue_max = if blue > blue_sat { blue } else { blue_sat };

    match column_index {
        0 | 1 => Color::Rgb(red, ambient, blue_sat),
        2..=4 => Color::Rgb(red, green / 2, blue_sat),
        5 | 6 => Color::Rgb(red, green, blue_sat),
        7 | 8 => Color::Rgb(ambient, green, blue_sat),
        9 | 10 => Color::Rgb(ambient, ambient, blue_max),
        11..=13 => Color::Rgb(blue, ambient, blue_max),
        14 => Color::Rgb(red, ambient, blue_max),
        15.. => Color::Reset,
    }
}

type ColorTable = Vec<Vec<Color, 15>, 60>;

static CORE1_STACK: StaticCell<Stack<16384>> = StaticCell::new();
static COLOR_BUFFER: StaticCell<[ColorTable; 8]> = StaticCell::new();
static COLOR_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, ColorTable>> = StaticCell::new();
static ADVANCE_FRAME: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static IS_PAUSED: Mutex<CriticalSectionRawMutex, bool> = Mutex::new(false);

#[embassy_executor::task]
async fn color_task(mut sender: Sender<'static, CriticalSectionRawMutex, ColorTable>) -> ! {
    let mut ticks: usize = 0;

    let create_color_row =
        |row_index| (0..15).map(move |column_index| get_color(row_index, column_index));

    let create_color_rows =
        |ticks| (0..ticks).map(move |row_index| create_color_row(ticks - row_index).collect());

    loop {
        let buffer = sender.send().await;
        buffer.clear();

        if ticks % 60 < 30 {
            buffer.extend(create_color_rows(ticks % 30));
        } else {
            buffer.extend(iter::repeat_n(Vec::new(), ticks % 30));
            buffer.extend(create_color_rows(30));
        }

        sender.send_done();

        ticks = ticks.wrapping_add(1);
    }
}

#[embassy_executor::task]
async fn button_task(mut input: Input<'static>) -> ! {
    loop {
        let button_up_and_then_down = async {
            input.wait_for_high().await;
            input.wait_for_low().await;
        };

        button_up_and_then_down.await;

        let button_up_or_timeout = with_timeout(Duration::from_millis(500), input.wait_for_high());

        let is_short_press = button_up_or_timeout.await.is_ok();
        if is_short_press {
            // Continue to draw a single frame
            ADVANCE_FRAME.signal(());
        } else {
            let mut guard = IS_PAUSED.lock().await;

            let is_paused = *guard;
            *guard = !is_paused;

            let is_paused = *guard;
            if !is_paused {
                // Continue drawing frames
                ADVANCE_FRAME.signal(());
            }
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // Kasuari necessitates the configuration of a global allocator
    esp_alloc::heap_allocator!(size: 32 * 1024);

    // Crank up the CPU clock frequency to 240 MHz
    let config = Config::default().with_cpu_clock(CpuClock::max());

    // Peripherals of ESP32
    let p = esp_hal::init(config);

    // When idle, this core will wait for an interrupt
    let timg0 = timg::TimerGroup::new(p.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Set up a channel for sending tables of colors without copying
    let buffer = COLOR_BUFFER.init(array::from_fn(|_| Vec::new()));
    let channel = COLOR_CHANNEL.init(Channel::new(buffer));
    let (sender, mut receiver) = channel.split();

    // Grab the single button for user-defined behavior
    // External pull-up resistor installed, so don't enable any more
    let input_config = InputConfig::default().with_pull(Pull::None);
    let input = Input::new(p.GPIO14, input_config);

    // Utilize the second CPU core for graphics
    let control = SoftwareInterruptControl::new(p.SW_INTERRUPT);
    let stack = CORE1_STACK.init(Stack::new());
    esp_rtos::start_second_core(
        p.CPU_CTRL,
        control.software_interrupt0,
        control.software_interrupt1,
        stack,
        move || {
            // Tall 17:32 TFT-LCD on an 8-bit bus
            let rst = Output::new(p.GPIO5, Level::High, Default::default());
            let _cs = Output::new(p.GPIO6, Level::Low, Default::default());
            let dc = Output::new(p.GPIO7, Level::Low, Default::default());
            let wr = Output::new(p.GPIO8, Level::High, Default::default());
            let _rd = Output::new(p.GPIO9, Level::High, Default::default());
            let _pwr = Output::new(p.GPIO15, Level::High, Default::default());
            let _blk = Output::new(p.GPIO38, Level::High, Default::default());
            let d0 = Output::new(p.GPIO39, Level::Low, Default::default());
            let d1 = Output::new(p.GPIO40, Level::Low, Default::default());
            let d2 = Output::new(p.GPIO41, Level::Low, Default::default());
            let d3 = Output::new(p.GPIO42, Level::Low, Default::default());
            let d4 = Output::new(p.GPIO45, Level::Low, Default::default());
            let d5 = Output::new(p.GPIO46, Level::Low, Default::default());
            let d6 = Output::new(p.GPIO47, Level::Low, Default::default());
            let d7 = Output::new(p.GPIO48, Level::Low, Default::default());
            let bus = Generic8BitBus::new((d0, d1, d2, d3, d4, d5, d6, d7));
            let interface = ParallelInterface::new(bus, dc, wr);
            let mut display = Builder::new(ST7789, interface)
                .reset_pin(rst)
                .invert_colors(ColorInversion::Inverted)
                .display_size(170, 320)
                .display_offset(35, 0)
                .init(&mut Delay)
                .unwrap();

            // No executor or tasks on the second CPU core in this example
            let mut backend = DumoBackend::new(&mut display, &FONT_10X30_4_BITS);
            backend.fg_reset = Some(FG_COLOR.into());
            backend.bg_reset = Some(BG_COLOR.into());

            // Hand the Dumo backend over to Ratatui
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.clear().unwrap();

            loop {
                let is_paused = IS_PAUSED.try_lock().is_ok_and(|guard| *guard);
                if is_paused {
                    // No executor, so it's okay to block this core
                    block_on(ADVANCE_FRAME.wait());
                }

                let render = |frame: &mut Frame| {
                    let center = frame.area().centered_horizontally(Length(15));

                    // Until there are new colors available, this core waits here
                    let row_major_colors = block_on(receiver.receive());
                    let row_count = row_major_colors.len();

                    for (row, row_colors) in center.rows().zip(row_major_colors) {
                        if row_colors.is_empty() {
                            continue;
                        }

                        for (column, color) in row.columns().zip(row_colors) {
                            frame.render_widget(Block::new().style(*color), column);
                        }

                        frame.render_widget(RatatuiLogo::tiny(), row);
                    }

                    receiver.receive_done();

                    if let Ok(y_offset) = row_count.try_into() {
                        let logo_area = center.offset(Offset::new(0, y_offset));

                        frame.render_widget(RatatuiLogo::tiny(), logo_area);
                    }
                };

                terminal.draw(render).unwrap();
            }
        },
    );

    spawner.spawn(color_task(sender)).unwrap();
    spawner.spawn(button_task(input)).unwrap();
}
