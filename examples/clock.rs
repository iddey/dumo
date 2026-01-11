use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread;
use std::time::Duration;

use dumo::DumoBackend;
use dumo::error::Error;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::SimulatorEvent::{KeyDown, Quit};
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::{Color, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::RatatuiLogo;
use ratatui::widgets::canvas::{Canvas, Circle};
use ratatui::{Frame, Terminal};
use tui_big_text::{BigText, PixelSize};

/// Displays a clock, rendering the digits in the hours, the minutes, and the seconds.
pub fn main() -> Result<(), Error<Infallible>> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_font = dumo::font_6x16!(4, '▀'..='▟', '🬀'..='🬻', '𜴀'..='𜷥', [" 𜺨🮂𜺫🯦🯧𜺣𜺠🮅"]);

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    let mut window = Window::new("Simulator", &output_settings);

    let is_running = AtomicBool::new(true);
    let base_index = AtomicU8::new(180);

    let mut backend = DumoBackend::new(&mut display, &bitmap_font);

    backend.fg_reset = Some(Rgb565::new(30, 60, 30));

    let backend = backend.with_flush(|display| {
        window.update(display);

        for event in window.events() {
            match event {
                KeyDown {
                    keycode: Keycode::SPACE | Keycode::RETURN,
                    ..
                } => {
                    let _ = base_index.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |index| {
                        let index = match index {
                            ..216 if index % 36 == 21 => index + 15,
                            ..216 if index % 6 == 3 => index + 3,
                            ..225 => index + 1,
                            225.. => 0,
                        };

                        Some(index)
                    });
                }
                Quit => {
                    is_running.store(false, Ordering::Relaxed);
                }
                _ => continue,
            }
        }

        Ok(())
    });

    let mut terminal = Terminal::new(backend)?;

    let horizontal = Layout::horizontal([Constraint::Length(4); 8]);

    let vertical = Layout::vertical([Constraint::Ratio(3, 7), Constraint::Ratio(4, 7)]);

    let pixel_sizes = [
        PixelSize::Octant,
        PixelSize::Sextant,
        PixelSize::Sextant,
        PixelSize::Quadrant,
        PixelSize::Quadrant,
        PixelSize::Sextant,
        PixelSize::Sextant,
        PixelSize::Octant,
    ];

    let offsets = [
        [0, 0],
        [1, 0],
        [2, 0],
        [2, 1],
        [2, 2],
        [1, 2],
        [0, 2],
        [0, 1],
    ];

    let offsets = offsets.map(|[offset, offset_by_six]| 6 * offset_by_six + offset + 16);

    let mut ticks = 0;

    while is_running.load(Ordering::Relaxed) {
        let digits = chrono::Local::now().format("%H:%M:%S").to_string();

        let base_index = base_index.load(Ordering::SeqCst);

        let big_text = |(digit_index, (char, pixel_size)): (usize, (char, _))| {
            let offset = offsets[(digit_index + ticks) % offsets.len()];

            let span = char.fg(Color::Indexed(base_index + offset));
            let line = span.into();

            BigText::builder()
                .pixel_size(pixel_size)
                .lines([line])
                .build()
        };

        let big_digits = digits.chars().zip(pixel_sizes).enumerate().map(big_text);

        let circle = Canvas::default()
            .marker(Marker::HalfBlock)
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| ctx.draw(&Circle::new(0.0, 0.0, 1.0, Color::Reset)));

        let render = |frame: &mut Frame| {
            let area = frame.area();

            frame.render_widget(circle, area);

            let [above_divide, below_divide] = area
                .centered(Constraint::Length(32), Constraint::Length(7))
                .layout(&vertical);

            frame.render_widget(
                RatatuiLogo::tiny(),
                above_divide.centered_horizontally(Constraint::Length(15)),
            );

            let areas: [_; 8] = below_divide.layout(&horizontal);

            for (area, big_digit) in areas.into_iter().zip(big_digits) {
                frame.render_widget(big_digit, area);
            }
        };

        terminal.draw(render)?;

        thread::sleep(Duration::from_millis(50));

        ticks = ticks.wrapping_add(1);
    }

    Ok(())
}
