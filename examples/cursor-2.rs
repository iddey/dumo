use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use std::time::Duration;

use dumo::DumoBackend;
use dumo::blink::{Blink, Blinked, ControlCursorBlinking};
use dumo::cursor::Cursor;
use dumo::error::Error;
use dumo::fonts::*;
use embedded_graphics::geometry::AnchorX;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::SimulatorEvent::{KeyDown, Quit};
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Offset};
use ratatui::style::{Modifier, Stylize};
use ratatui::symbols::scrollbar;
use ratatui::text::Text;
use ratatui::widgets::ScrollbarOrientation::{HorizontalBottom, VerticalRight};
use ratatui::widgets::{Paragraph, ScrollDirection, Scrollbar, ScrollbarState};
use ratatui::{Frame, Terminal};

/// Displays the tables of supported glyphs that are enabled by default, with additional sets being
/// available, where moving the cursor against the edge of the text area scrolls its content.
pub fn main() -> Result<(), Error<Infallible>> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    let mut window = Window::new("Simulator", &output_settings);

    let is_running = AtomicBool::new(true);
    let is_interact = AtomicBool::new(false);
    let is_change_text = AtomicBool::new(false);
    let is_toggle_bold = AtomicBool::new(false);
    let delta_x = AtomicI32::new(0);
    let delta_y = AtomicI32::new(0);

    let mut backend = DumoBackend::new(&mut display, &FONT_8X24_4_BITS);
    backend.font_bold = Some(&FONT_8X24_BOLD_4_BITS);
    backend.fg_reset = Some(Rgb565::new(30, 60, 30));
    backend.bg_reset = Some(Rgb565::CSS_INDIGO);
    backend.anchor_x = AnchorX::Center;

    let backend = backend
        .with_cursor(Cursor::default().blink(Blink::with_period(10)))
        .with_flush(|display| {
            window.update(display);

            for event in window.events() {
                match event {
                    KeyDown {
                        keycode, keymod, ..
                    } => {
                        is_interact.store(true, Ordering::Relaxed);

                        match keycode {
                            Keycode::RIGHT | Keycode::L if keymod == Mod::NOMOD => {
                                delta_x.fetch_add(1, Ordering::SeqCst);
                            }
                            Keycode::LEFT | Keycode::H if keymod == Mod::NOMOD => {
                                delta_x.fetch_sub(1, Ordering::SeqCst);
                            }
                            Keycode::DOWN | Keycode::J if keymod == Mod::NOMOD => {
                                delta_y.fetch_add(1, Ordering::SeqCst);
                            }
                            Keycode::UP | Keycode::K if keymod == Mod::NOMOD => {
                                delta_y.fetch_sub(1, Ordering::SeqCst);
                            }
                            Keycode::SPACE | Keycode::RETURN => {
                                is_change_text.store(true, Ordering::Relaxed);
                            }
                            Keycode::TAB => {
                                is_toggle_bold.store(true, Ordering::Relaxed);
                            }
                            _ => continue,
                        };
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

    terminal.clear()?;

    let texts = [
        Text::from(include_str!("../glyph-subsets/latin.set")),
        Text::from(include_str!("../glyph-subsets/tui-block.set")),
        Text::from(include_str!("../glyph-subsets/tui-boxes.set")),
        Text::from(include_str!("../glyph-subsets/tui-dots-2x4.set")),
        Text::from(include_str!("../glyph-subsets/tui-rect-2x3.set")),
        Text::from(include_str!("../glyph-subsets/tui-rect-2x4.set")),
    ];

    let mut texts = texts.into_iter().cycle();
    let mut text = texts.next().expect("expected text");

    let mut vertical_scroll_state = ScrollbarState::default();
    let mut horizontal_scroll_state = ScrollbarState::default();

    while is_running.load(Ordering::Relaxed) {
        let is_interact = is_interact
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default();

        let is_change_text = is_change_text
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default();

        let is_toggle_bold = is_toggle_bold
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default();

        if is_interact {
            terminal
                .backend_mut()
                .advance_cursor_blink_to(Blinked(true))?;
        }

        if is_change_text {
            text = texts.next().expect("expected text");
        }

        if is_toggle_bold {
            if text.style.has_modifier(Modifier::BOLD) {
                text = text.not_bold();
            } else {
                text = text.bold();
            }
        }

        let delta_x = delta_x
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(0))
            .unwrap_or_default()
            .clamp(-1, 1);

        let delta_y = delta_y
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(0))
            .unwrap_or_default()
            .clamp(-1, 1);

        let offset = Offset::new(delta_x, delta_y);
        let cursor_position = terminal.get_cursor_position()?;

        let render = |frame: &mut Frame| {
            let area = frame.area();

            let [left, right] = Layout::horizontal([Fill(1), Length(1)]).areas(area);
            let [top, bottom] = Layout::vertical([Fill(1), Length(1)]).areas(area);

            let text_area = top.intersection(left);

            let vertical_positions = text
                .height()
                .saturating_sub(text_area.height.into())
                .saturating_add(1);

            let horizontal_positions = text
                .width()
                .saturating_sub(text_area.width.into())
                .saturating_add(1);

            vertical_scroll_state = vertical_scroll_state.content_length(vertical_positions);
            horizontal_scroll_state = horizontal_scroll_state.content_length(horizontal_positions);

            if cursor_position.y == text_area.top() && delta_y < 0 {
                vertical_scroll_state.scroll(ScrollDirection::Backward);
            } else if let Some(bottom) = text_area.bottom().checked_sub(1)
                && cursor_position.y == bottom
                && delta_y > 0
            {
                vertical_scroll_state.scroll(ScrollDirection::Forward);
            }

            if cursor_position.x == text_area.left() && delta_x < 0 {
                horizontal_scroll_state.scroll(ScrollDirection::Backward);
            } else if let Some(right) = text_area.right().checked_sub(1)
                && cursor_position.x == right
                && delta_x > 0
            {
                horizontal_scroll_state.scroll(ScrollDirection::Forward);
            }

            if text_area.contains(cursor_position.offset(offset)) {
                frame.set_cursor_position(cursor_position.offset(offset));
            } else {
                frame.set_cursor_position(cursor_position);
            }

            let symbols = scrollbar::Set {
                track: "░",
                thumb: "▓",
                ..Default::default()
            };

            let vertical_scrollbar = Scrollbar::new(VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .symbols(symbols.clone());

            let horizontal_scrollbar = Scrollbar::new(HorizontalBottom)
                .begin_symbol(None)
                .end_symbol(None)
                .symbols(symbols);

            frame.render_stateful_widget(vertical_scrollbar, top, &mut vertical_scroll_state);
            frame.render_stateful_widget(horizontal_scrollbar, left, &mut horizontal_scroll_state);
            frame.render_widget("▒", bottom.intersection(right));

            let text = text.clone();

            let vertical_scroll = vertical_scroll_state
                .get_position()
                .try_into()
                .unwrap_or(u16::MAX);

            let horizontal_scroll = horizontal_scroll_state
                .get_position()
                .try_into()
                .unwrap_or(u16::MAX);

            let paragraph = Paragraph::new(text).scroll((vertical_scroll, horizontal_scroll));

            frame.render_widget(paragraph, text_area);
        };

        terminal.draw(render)?;

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
