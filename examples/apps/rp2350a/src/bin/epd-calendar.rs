#![no_std]
#![no_main]

use core::cell::RefCell;
use core::ptr::addr_of_mut;

use cortex_m::asm;
use dumo::DumoBackend;
use dumo::fonts::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_time::Delay;
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use ratatui::layout::Constraint::Length;
use ratatui::prelude::*;
use ratatui::symbols::border::{QUADRANT_OUTSIDE, Set};
use ratatui::text::ToLine;
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, List};
use ssd1619a::interface::SpiInterface;
use ssd1619a::{BUFFER_SIZE, Builder};
use static_cell::{ConstStaticCell, StaticCell};
use time::{Date, Month, Time};
use tui_big_text::{BigTextBuilder, PixelSize};

type SpiDriver = spi::Spi<'static, SPI1, spi::Blocking>;

static mut CORE1_STACK: Stack<8192> = Stack::new();
static SPI_BUS: StaticCell<NoopMutex<RefCell<SpiDriver>>> = StaticCell::new();
static FRAMEBUFFER: ConstStaticCell<[u8; BUFFER_SIZE]> = ConstStaticCell::new([0x00; _]);

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[cortex_m_rt::entry]
fn main() -> ! {
    unsafe {
        // Configuration of a global allocator
        embedded_alloc::init!(HEAP, 128 * 1024);
    }

    // Peripherals of RP2350A
    let p = embassy_rp::init(Default::default());

    // Shared SPI bus
    let spi = spi::Spi::new_blocking_txonly(p.SPI1, p.PIN_10, p.PIN_11, Default::default());
    let spi_bus = SPI_BUS.init(NoopMutex::new(RefCell::new(spi)));

    // Utilize the second CPU core for graphics
    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            // Diminishing returns over 16 MHz
            let mut config = spi::Config::default();
            config.frequency = 16_000_000;

            let cs = Output::new(p.PIN_9, Level::High);
            let dc = Output::new(p.PIN_8, Level::Low);
            let rst = Output::new(p.PIN_12, Level::High);
            let busy = Input::new(p.PIN_13, Pull::None);
            let spi_device = SpiDeviceWithConfig::new(spi_bus, cs, config);
            let interface = SpiInterface::new(spi_device, dc, rst, busy);
            let mut display = Builder::new(interface)
                .with_buffer(FRAMEBUFFER.take())
                .init(&mut Delay)
                .unwrap();

            // No executor or tasks on the second CPU core in this example
            let backend = DumoBackend::new(&mut display, &FONT_12X30_1_BIT)
                .with_flush(|display| display.update(&mut Delay));

            // Hand the Dumo backend over to Ratatui
            let terminal = Terminal::new(backend).unwrap();

            // Run the app, which is stateless
            App::run(terminal)
        },
    );

    loop {
        asm::nop();
    }
}

struct App;

impl App {
    fn run(mut terminal: Terminal<impl Backend>) -> ! {
        terminal
            .draw(|frame: &mut Frame| frame.render_widget(Self, frame.area()))
            .unwrap();

        // Block the core running the app for 10 seconds
        Delay.delay_ms(10_000);

        terminal.backend_mut().clear().unwrap();
        terminal.backend_mut().flush().unwrap();

        loop {
            asm::nop();
        }
    }
}

impl Widget for App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [area] = area.layout(&Layout::horizontal([Length(33)]));

        let horizontal = Layout::horizontal([Length(11), Length(22)]);
        let [events_area, date_time_area] = horizontal.areas(area);

        let vertical = Layout::vertical([Length(3), Length(7)]);
        let [upper_right_area, calendar_area] = vertical.areas(date_time_area);

        let clock_area = upper_right_area.centered_horizontally(Length(19));

        render_clock(clock_area, buffer);
        render_events(events_area, buffer);
        render_calendar(calendar_area, buffer);
    }
}

fn render_clock(area: Rect, buffer: &mut Buffer) {
    let current_time = Time::from_hms(11, 48, 00).unwrap();

    BigTextBuilder::default()
        .pixel_size(PixelSize::Sextant)
        .lines([current_time.to_line()])
        .build()
        .render(area, buffer);
}

fn render_events(area: Rect, buffer: &mut Buffer) {
    let items = [
        Text::from_iter(["🯦12:30 PM", "Lunch"]),
        Text::from_iter(["🯦 4:30 PM", "Movies"]),
        Text::from_iter(["🯦 7:00 PM", "Ratatui"]),
    ];

    let items = items.map(|text| text.right_aligned());

    let block = Block::bordered()
        .title(Line::from("Today").reversed())
        .border_set(Set {
            top_left: "█",
            top_right: "█",
            horizontal_top: "█",
            ..QUADRANT_OUTSIDE
        });

    Widget::render(List::new(items).block(block), area, buffer);
}

fn render_calendar(area: Rect, buffer: &mut Buffer) {
    let current_date = Date::from_calendar_date(2007, Month::June, 22).unwrap();

    let event_dates = [
        Date::from_calendar_date(2007, Month::June, 13).unwrap(),
        Date::from_calendar_date(2007, Month::June, 22).unwrap(),
        Date::from_calendar_date(2007, Month::June, 27).unwrap(),
        Date::from_calendar_date(2007, Month::June, 29).unwrap(),
    ];

    let mut events = CalendarEventStore::default();

    for date in event_dates {
        let mut style = Style::new().underlined();

        if date == current_date {
            style = style.add_modifier(Modifier::REVERSED);
        }

        events.add(date, style);
    }

    Monthly::new(current_date, events)
        .show_month_header(Style::new().reversed())
        .show_weekdays_header(Style::new())
        .render(area, buffer);
}
