use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use dumo::color::Palettes;
use dumo::error::Error;
use dumo::{ConfigureBackend, DumoBackend};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::SimulatorEvent::{KeyDown, Quit};
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ratatui::layout::Constraint;
use ratatui::layout::{Layout, Rect};
use ratatui::style::Color;
use ratatui::{Frame, Terminal};

/// Displays how the built-in palettes translate the 4 or 8-bit color codes into 24-bit color codes.
pub fn main() -> Result<(), Error<Infallible>> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_font = dumo::font_6x16!(8, ["█"]);

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    let mut window = Window::new("Simulator", &output_settings);

    let is_running = AtomicBool::new(true);
    let is_advance = AtomicBool::new(false);

    let backend = DumoBackend::new(&mut display, &bitmap_font);
    let backend = backend.with_flush(|display| {
        window.update(display);

        for event in window.events() {
            match event {
                KeyDown {
                    keycode: Keycode::SPACE | Keycode::RETURN,
                    ..
                } => {
                    is_advance.store(true, Ordering::Relaxed);
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

    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(30),
        Constraint::Fill(1),
        Constraint::Percentage(30),
        Constraint::Fill(1),
        Constraint::Percentage(30),
        Constraint::Fill(1),
    ]);

    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Percentage(40),
        Constraint::Percentage(40),
        Constraint::Fill(1),
    ]);

    let palettes = [
        Rgb888::WIN_16,
        Rgb888::WEB_256,
        Rgb888::XTERM_16,
        Rgb888::XTERM_256,
    ];

    let mut palettes = palettes.into_iter().cycle();

    while is_running.load(Ordering::Relaxed) {
        let is_advance = is_advance
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default();

        if is_advance {
            let palette = palettes.next().expect("expected palette");

            terminal.backend_mut().set_palette(palette);
            terminal.swap_buffers();
        }

        let render = |frame: &mut Frame| {
            let area = frame.area();

            let wide_areas = area.layout(&vertical);
            let tall_areas = area.layout(&horizontal);

            let [upper_head, lower_head, upper_body, lower_body, tail] = wide_areas;

            let subdivide = |wide| tall_areas.map(|tall| tall.intersection(wide));

            let head = colors_16([upper_head, lower_head]);
            let body = colors_216([upper_body, lower_body], subdivide);
            let tail = gray_values(tail, subdivide);

            let buffer = frame.buffer_mut();

            for (area, color) in head.into_iter().chain(body).chain(tail) {
                for position in area.positions() {
                    buffer[position].set_symbol("█").set_fg(color);
                }
            }
        };

        terminal.draw(render)?;

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn colors_16([upper_head, lower_head]: [Rect; 2]) -> impl IntoIterator<Item = (Rect, Color)> {
    let head = upper_head.union(lower_head);

    let eighths: [_; 8] = head.layout(&Layout::horizontal([Constraint::Fill(1); 8]));

    let areas = [upper_head, lower_head]
        .map(|wide| eighths.map(|tall| tall.intersection(wide)))
        .into_iter()
        .flatten();

    let colors = (0..16).map(Color::Indexed);

    areas.zip(colors)
}

fn colors_216(
    [upper_body, lower_body]: [Rect; 2],
    subdivide: impl Fn(Rect) -> [Rect; 7],
) -> impl IntoIterator<Item = (Rect, Color)> {
    let body = upper_body.union(lower_body);

    let [a, left, b, center, c, right, d] = subdivide(body);

    let wide_sixths = [upper_body, lower_body]
        .map(|block| -> [_; 6] { block.layout(&Layout::vertical([Constraint::Fill(1); 6])) });

    let tall_sixths = [left, center, right]
        .map(|block| -> [_; 6] { block.layout(&Layout::horizontal([Constraint::Fill(1); 6])) });

    let areas = wide_sixths
        .map(|wide_sixth| {
            tall_sixths.map(|tall_sixth| {
                wide_sixth.map(|wide| tall_sixth.map(|tall| tall.intersection(wide)))
            })
        })
        .into_iter()
        .flatten()
        .flatten()
        .flatten();

    let colors = (16..232).map(Color::Indexed);

    let spacers = [a, b, c, d].into_iter().zip([
        Color::Black,
        Color::DarkGray,
        Color::DarkGray,
        Color::White,
    ]);

    areas.zip(colors).chain(spacers)
}

fn gray_values(
    tail: Rect,
    subdivide: impl Fn(Rect) -> [Rect; 7],
) -> impl IntoIterator<Item = (Rect, Color)> {
    let [a, left, b, center, c, right, d] = subdivide(tail);

    let areas = [left, right]
        .map(|block| -> [_; 12] { block.layout(&Layout::horizontal([Constraint::Fill(1); 12])) })
        .into_iter()
        .flatten();

    let colors = (232..=255).map(Color::Indexed);

    let spacers = [a, b, center, c, d].into_iter().zip([
        Color::Black,
        Color::DarkGray,
        Color::DarkGray,
        Color::DarkGray,
        Color::White,
    ]);

    areas.zip(colors).chain(spacers)
}
