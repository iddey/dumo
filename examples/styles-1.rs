use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use dumo::DumoBackend;
use dumo::blink::Blink;
use dumo::error::Error;
use embedded_graphics::geometry::AnchorX;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::SimulatorEvent::{KeyDown, Quit};
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Layout;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListState, Paragraph};
use ratatui::{Frame, Terminal};

/// Displays how styling text is possible with the help of one or more modifiers, using the letters
/// of the English alphabet as an example.
#[mplusfonts::strings]
pub fn main() -> Result<(), Error<Infallible>> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    #[strings::emit]
    let bitmap_font = dumo::font_8x24!(4, ["_|─│┘┐┌└"]);
    #[strings::emit]
    let bitmap_font_bold = dumo::font_8x24_bold!(4);

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    #[strings::skip]
    let mut window = Window::new("Simulator", &output_settings);

    let is_running = AtomicBool::new(true);
    let is_select_next = AtomicBool::new(false);
    let is_select_previous = AtomicBool::new(false);
    let is_toggle_selected = AtomicBool::new(false);

    let mut backend = DumoBackend::new(&mut display, &bitmap_font);

    backend.font_bold = Some(&bitmap_font_bold);
    backend.fg_reset = Some(Rgb565::new(30, 60, 30));
    backend.anchor_x = AnchorX::Right;

    let backend = backend.with_blink(Blink::with_period(16), Blink::with_period(8));
    let backend = backend.with_flush(|display| {
        window.update(display);

        for event in window.events() {
            match event {
                KeyDown {
                    keycode: Keycode::DOWN | Keycode::J | Keycode::TAB,
                    keymod: Mod::NOMOD,
                    ..
                } => is_select_next.store(true, Ordering::Relaxed),
                KeyDown {
                    keycode: Keycode::UP | Keycode::K,
                    keymod: Mod::NOMOD,
                    ..
                }
                | KeyDown {
                    keycode: Keycode::TAB,
                    keymod: Mod::LSHIFTMOD | Mod::RSHIFTMOD,
                    ..
                } => is_select_previous.store(true, Ordering::Relaxed),
                KeyDown {
                    keycode: Keycode::SPACE | Keycode::RETURN,
                    ..
                } => is_toggle_selected.store(true, Ordering::Relaxed),
                Quit => {
                    is_running.store(false, Ordering::Relaxed);
                }
                _ => continue,
            }
        }

        Ok(())
    });

    let mut terminal = Terminal::new(backend)?;

    let modifiers = Modifier::all().difference(Modifier::ITALIC);

    let mut state = ListState::default().with_selected(modifiers.into_iter().next().map(|_| 0));
    let mut style = Style::new();

    while is_running.load(Ordering::Relaxed) {
        is_select_next
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default()
            .then(|| state.select_next());

        is_select_previous
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default()
            .then(|| state.select_previous());

        let modifier = is_toggle_selected
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default()
            .then(|| state.selected().map(|n| modifiers.into_iter().nth(n)))
            .flatten()
            .flatten();

        if let Some(modifier) = modifier {
            style.add_modifier = if style.has_modifier(modifier) {
                style.add_modifier.difference(modifier)
            } else {
                style.add_modifier.union(modifier)
            };
        }

        let list = List::new(modifiers.iter_names().map(|(name, _)| name))
            .block(Block::bordered().title(Line::from("Modifiers").centered()))
            .highlight_style(Style::new().bg(Color::Indexed(240)))
            .highlight_symbol("▶ ")
            .scroll_padding(1);

        let paragraph = Paragraph::new(Span::styled("ABCDEFGHIJKLMNOPQRSTUVWXYZ", style))
            .block(Block::bordered().title(Line::from("Styled Text").centered()))
            .centered();

        let string: String = format!("{modifier:?}", modifier = style.add_modifier)
            .split_ascii_whitespace()
            .flat_map(|s| s.split("_").filter_map(|s| s.chars().next()))
            .collect();

        let render = |frame: &mut Frame| {
            let area = frame.area();

            let [top, middle, bottom] = Layout::vertical([60, 30, 10].map(Percentage)).areas(area);

            frame.render_stateful_widget(list, top, &mut state);
            frame.render_widget(paragraph, middle);
            frame.render_widget(string, bottom.centered_horizontally(Percentage(60)));
        };

        terminal.draw(render)?;

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
