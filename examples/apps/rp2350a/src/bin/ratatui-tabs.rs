//! Module pin-compatible with the Raspberry Pi Pico series (it has an RP2350A)
//! Featuring a display with a resolution of 80x160 and an ST7735S controller

#![no_std]
#![no_main]

use core::cell::RefCell;
use core::ptr::addr_of_mut;

use dumo::DumoBackend;
use dumo::fonts::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_futures::select::Either::{First, Second};
use embassy_futures::select::select;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Delay, Duration, Ticker};
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::prelude::Dimensions;
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7735s;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use panic_halt as _;
use ratatui::layout::Constraint::{Length, Min};
use ratatui::prelude::*;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Block, Padding, Paragraph, Tabs, Wrap};
use static_cell::StaticCell;
use strum::{Display, EnumIter, IntoEnumIterator, IntoStaticStr};

type SpiDriver = spi::Spi<'static, SPI1, spi::Blocking>;

static mut CORE1_STACK: Stack<8192> = Stack::new();
static SPI_BUS: StaticCell<NoopMutex<RefCell<SpiDriver>>> = StaticCell::new();
static LCD_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static SELECT_NEXT_TAB: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static CHANGE_OFFSET: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::task]
async fn select_next_tab_task(mut ticker: Ticker) {
    loop {
        ticker.next().await;

        SELECT_NEXT_TAB.signal(());
    }
}

#[embassy_executor::task]
async fn change_offset_task(mut ticker: Ticker) {
    loop {
        ticker.next().await;

        CHANGE_OFFSET.signal(());
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    unsafe {
        // Configuration of a global allocator
        embedded_alloc::init!(HEAP, 32 * 1024);
    }

    // Peripherals of RP2350A
    let p = embassy_rp::init(Default::default());

    // Shared SPI bus
    let spi = spi::Spi::new_blocking_txonly(p.SPI1, p.PIN_10, p.PIN_11, Default::default());
    let spi_bus = SPI_BUS.init(NoopMutex::new(RefCell::new(spi)));

    // Switch to a new tab every 6.25 seconds
    let slow_ticker = Ticker::every(Duration::from_millis(6250));
    let rapid_ticker = Ticker::every(Duration::from_millis(1250));

    // Utilize the second CPU core for graphics
    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            // Diminishing returns over 16 MHz
            let mut config = spi::Config::default();
            config.frequency = 16_000_000;

            // Wide 2:1 TFT-LCD on shared SPI bus
            let buffer = LCD_BUFFER.init([0; _]);
            let cs = Output::new(p.PIN_9, Level::High);
            let dc = Output::new(p.PIN_8, Level::Low);
            let rst = Output::new(p.PIN_12, Level::High);
            let _blk = Output::new(p.PIN_25, Level::High);
            let spi_device = SpiDeviceWithConfig::new(spi_bus, cs, config);
            let interface = SpiInterface::new(spi_device, dc, buffer);
            let mut display = Builder::new(ST7735s, interface)
                .reset_pin(rst)
                .color_order(ColorOrder::Bgr)
                .invert_colors(ColorInversion::Inverted)
                .orientation(Orientation::new().rotate(Rotation::Deg90))
                .display_size(80, 160)
                .display_offset(26, 1)
                .init(&mut Delay)
                .unwrap();

            let left = display
                .bounding_box()
                .resized((2, 80).into(), AnchorPoint::CenterLeft);

            let right = display
                .bounding_box()
                .resized((2, 80).into(), AnchorPoint::CenterRight);

            for area in [left, right] {
                display.fill_solid(&area, Default::default()).unwrap();
            }

            let text_area = display
                .bounding_box()
                .resized((156, 80).into(), AnchorPoint::Center);

            let mut display = display.cropped(&text_area);

            // No executor or tasks on the second CPU core in this example
            let backend = DumoBackend::new(&mut display, &FONT_6X16_4_BITS);

            // Hand the Dumo backend over to Ratatui
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.clear().unwrap();

            // Start a new app, rendering widgets and handling events
            App::default().run(terminal)
        },
    );

    spawner.spawn(select_next_tab_task(slow_ticker)).unwrap();
    spawner.spawn(change_offset_task(rapid_ticker)).unwrap();
}

#[derive(Default)]
struct App {
    selected_tab: SelectedTab,
    offset: (u16, u16),
}

#[derive(Default, Clone, Copy, Display, EnumIter, IntoStaticStr)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = " 1 ")]
    Tab1,
    #[strum(to_string = " 2 ")]
    Tab2,
    #[strum(to_string = " 3 ")]
    Tab3,
    #[strum(to_string = " 4 ")]
    Tab4 { offset: (u16, u16) },
}

impl App {
    fn run(mut self, mut terminal: Terminal<impl Backend>) -> ! {
        loop {
            terminal
                .draw(|frame| frame.render_widget(&self, frame.area()))
                .unwrap();

            // Until there is an event, the core running the app waits here
            block_on(self.handle_next_event());
        }
    }

    async fn handle_next_event(&mut self) {
        match select(SELECT_NEXT_TAB.wait(), CHANGE_OFFSET.wait()).await {
            First(_) => self.selected_tab = self.selected_tab.next(),
            Second(_) => {
                self.offset.1 += 8;
                self.offset.1 %= 16;

                if let SelectedTab::Tab4 { offset } = &mut self.selected_tab {
                    offset.0 += 2;
                    offset.0 %= 4;
                }
            }
        }
    }
}

impl SelectedTab {
    fn next(self) -> Self {
        match self {
            Self::Tab1 => Self::Tab2,
            Self::Tab2 => Self::Tab3,
            Self::Tab3 => Self::Tab4 { offset: (0, 0) },
            Self::Tab4 { .. } => Self::Tab1,
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let vertical = Layout::vertical([Length(1), Min(0)]);
        let [header_area, inner_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(8)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buffer, self.offset);
        self.render_tabs(tabs_area, buffer);
        self.selected_tab.render(inner_area, buffer);
    }
}

fn render_title(area: Rect, buffer: &mut Buffer, offset: (u16, u16)) {
    Paragraph::new("Ratatui Example")
        .scroll(offset)
        .render(area, buffer);
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        match self {
            Self::Tab1 => self.render_tab0(area, buffer),
            Self::Tab2 => self.render_tab1(area, buffer),
            Self::Tab3 => self.render_tab2(area, buffer),
            Self::Tab4 { offset } => self.render_tab3(area, buffer, offset),
        }
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buffer: &mut Buffer) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Default::default(), self.selected_tab.palette().c700);
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(self.selected_tab.index())
            .padding("", "")
            .divider(" ")
            .render(area, buffer);
    }
}

impl SelectedTab {
    fn title(self) -> Line<'static> {
        Into::<&'static str>::into(self)
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    fn render_tab0(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Hello, World!")
            .block(self.block())
            .render(area, buffer);
    }

    fn render_tab1(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Welcome to the Ratatui tabs example!")
            .block(self.block())
            .wrap(Wrap { trim: true })
            .render(area, buffer);
    }

    fn render_tab2(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Look! I'm different than others!")
            .block(self.block())
            .wrap(Wrap { trim: true })
            .render(area, buffer);
    }

    fn render_tab3(self, area: Rect, buffer: &mut Buffer, offset: (u16, u16)) {
        Paragraph::new("I know, these are some basic changes. But I think you got the main idea.")
            .block(self.block())
            .wrap(Wrap { trim: true })
            .scroll(offset)
            .render(area, buffer);
    }

    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700)
    }

    const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Tab1 => tailwind::BLUE,
            Self::Tab2 => tailwind::EMERALD,
            Self::Tab3 => tailwind::INDIGO,
            Self::Tab4 { .. } => tailwind::RED,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Tab1 => 0,
            Self::Tab2 => 1,
            Self::Tab3 => 2,
            Self::Tab4 { .. } => 3,
        }
    }
}
