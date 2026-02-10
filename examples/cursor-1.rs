use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use std::time::Duration;

use dumo::blink::{Blink, Blinked, ControlCursorBlinking};
use dumo::cursor::{Colors, Cursor, Extent, Symbol};
use dumo::error::{AdvanceCursorBlinkingError, Error, SetCursorError};
use dumo::{ConfigureBackend, ConfigureCursorWrapper, DumoBackend};
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::SimulatorEvent::{KeyDown, Quit};
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Margin, Offset, Position, Spacing};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, List, ListState};
use ratatui::{Frame, Terminal};

/// Demonstrates how the appearance of the cursor indicator, when enabled using the cursor wrapper,
/// can also be configured with methods that are available for a borrowed backend.
#[mplusfonts::strings]
pub fn main() -> Result<(), Error<Infallible>> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    #[strings::emit]
    let bitmap_font = dumo::font_8x20!(4, '0'..='9', 'A'..='Z', 'a'..='z', [r#"#"()"#], '─'..='╋');

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    #[strings::skip]
    let mut window = Window::new("Simulator", &output_settings);

    let is_running = AtomicBool::new(true);
    let is_interact = AtomicBool::new(false);
    let is_select = AtomicBool::new(false);
    let delta_x = AtomicI32::new(0);
    let delta_y = AtomicI32::new(0);

    let mut backend = DumoBackend::new(&mut display, &bitmap_font)
        .with_blink(Blink::with_period(16), Blink::with_period(8))
        .with_cursor(Cursor::default())
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
                                is_select.store(true, Ordering::Relaxed);
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

    backend.set_fg_reset(Some(Rgb565::new(30, 60, 30)));
    backend.set_bg_reset(Some(Rgb565::CSS_MIDNIGHT_BLUE));

    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let blinks = [Blink::with_period(0), Blink::with_period(10)];

    let colors = [
        Colors::ReversedReset,
        Colors::InvertedReset,
        Colors::Custom {
            fg: Color::Black,
            bg: Color::Rgb(
                Rgb888::CSS_ORANGE_RED.r(),
                Rgb888::CSS_ORANGE_RED.g(),
                Rgb888::CSS_ORANGE_RED.b(),
            ),
        },
        Colors::Custom {
            fg: Color::Rgb(
                Rgb888::CSS_MIDNIGHT_BLUE.r(),
                Rgb888::CSS_MIDNIGHT_BLUE.g(),
                Rgb888::CSS_MIDNIGHT_BLUE.b(),
            ),
            bg: Color::White,
        },
    ];

    let extents = [
        Extent::FullBlock,
        Extent::VerticalBar { width: 2 },
        Extent::Underline { height: 2 },
        Extent::Custom(Rectangle {
            top_left: Point::new(2, 8),
            size: Size::new(4, 4),
        }),
    ];

    let symbols = [Symbol::UnderCursor, Symbol::Custom("🮕🮕")];

    let mut list_states = [ListState::default().with_selected(Some(0)); 4];
    let mut ticks = 0;

    while is_running.load(Ordering::Relaxed) {
        const TICK_CAP: usize = 40;

        let is_first_half = ticks < TICK_CAP.div_ceil(2);

        let is_interact = is_interact
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default();

        let is_select = is_select
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
            .unwrap_or_default();

        if is_interact {
            terminal
                .backend_mut()
                .advance_cursor_blink_to(Blinked(true))
                .map_or_else(map_invalid_blinked_to_ok, Ok)?;
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

        let offset = fix_offset_from_position(offset, cursor_position, is_first_half);
        let cursor_position = cursor_position.offset(offset);

        terminal
            .set_cursor_position(cursor_position)
            .map_or_else(map_invalid_position_to_ok, Ok)?;

        terminal.show_cursor()?;

        let text = Text::from_iter([
            Line::from_iter([
                Span::from("Hello, "),
                Span::from("this is dim, ").dim(),
                Span::from("reversed, ").reversed(),
            ]),
            Line::from_iter([
                Span::from("underlined and ").underlined(),
                Span::from("hidden, ").underlined().hidden(),
                Span::from("blink, ").slow_blink(),
            ]),
            Line::from_iter([
                Span::from("すごい〜！"),
                if is_first_half {
                    Span::from("全角、")
                } else {
                    Span::from("zenkaku, ")
                },
                Span::from("取り消し線。").crossed_out(),
                Span::from("\u{3000}"),
            ]),
        ]);

        let blocks = ["Blink", "Colors", "Extent", "Symbol"].map(|title| {
            Block::bordered()
                .title(title)
                .merge_borders(MergeStrategy::Exact)
        });

        let lists = [
            List::from_iter(blinks.iter().map(|blink| {
                if let Blink::Repeat(0, 0) = blink {
                    String::from("SolidOn")
                } else {
                    format!("{blink:?}")
                }
            })),
            List::from_iter(colors.iter().map(|colors| {
                if let Colors::Custom { fg, bg } = colors {
                    format!("{fg}, {bg}")
                } else {
                    format!("{colors:?}")
                }
            })),
            List::from_iter(extents.iter().map(|extent| match extent {
                Extent::VerticalBar { width } => format!("VerticalBar {width}"),
                Extent::Underline { height } => format!("Underline {height}"),
                Extent::Custom(area) => format!(
                    "Rectangle {width}×{height}",
                    width = area.size.width,
                    height = area.size.height
                ),
                _ => format!("{extent:?}"),
            })),
            List::from_iter(symbols.iter().map(|symbol| format!("{symbol:?}"))),
        ];

        let lists = lists.map(|list| {
            list.style(Style::new().dim())
                .highlight_style(Style::new().not_dim())
        });

        let render = |frame: &mut Frame| {
            let area = frame.area();

            let [top, bottom] = Layout::vertical([Fill(1), Length(9)]).areas(area);

            frame.render_widget(text, top);

            let block_areas = Layout::horizontal([Fill(1); 2])
                .spacing(Spacing::Overlap(1))
                .areas::<2>(bottom)
                .into_iter()
                .zip([[4, 6], [6, 4]])
                .flat_map(|(area, rows)| {
                    Layout::vertical(rows.map(Length))
                        .spacing(Spacing::Overlap(1))
                        .areas::<2>(area)
                });

            block_areas
                .clone()
                .zip(blocks.iter().cloned())
                .zip(list_states.iter_mut().zip(lists.iter().cloned()))
                .map(|((area, block), (state, list))| (area, (state, list.block(block))))
                .filter(|(area, _)| !area.contains(cursor_position))
                .for_each(|(area, (state, list))| frame.render_stateful_widget(list, area, state));

            block_areas
                .clone()
                .zip(blocks.map(|block| block.border_type(BorderType::Thick)))
                .zip(list_states.iter_mut().zip(lists))
                .map(|((area, block), (state, list))| (area, (state, list.block(block))))
                .filter(|(area, _)| area.contains(cursor_position))
                .for_each(|(area, (state, list))| frame.render_stateful_widget(list, area, state));

            if is_select {
                for (index, area) in block_areas.enumerate() {
                    area.inner(Margin::new(1, 1))
                        .rows()
                        .enumerate()
                        .filter(|(_, area)| area.contains(cursor_position))
                        .for_each(|(row_index, _)| list_states[index].select(Some(row_index)));
                }
            }
        };

        terminal.draw(render)?;

        if is_select {
            if let Some(row_index) = list_states[0].selected() {
                terminal.backend_mut().set_cursor_blink(blinks[row_index]);
            }

            if let Some(row_index) = list_states[1].selected() {
                terminal.backend_mut().set_cursor_colors(colors[row_index]);
            }

            if let Some(row_index) = list_states[2].selected() {
                terminal.backend_mut().set_cursor_extent(extents[row_index]);
            }

            if let Some(row_index) = list_states[3].selected() {
                terminal.backend_mut().set_cursor_symbol(symbols[row_index]);
            }
        }

        ticks = ticks
            .wrapping_add(1)
            .checked_rem(TICK_CAP)
            .unwrap_or_default();

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn map_invalid_blinked_to_ok<T>(error: Error<T>) -> Result<(), Error<T>> {
    use AdvanceCursorBlinkingError::*;

    if let Error::AdvanceCursorBlinking(InvalidBlinked) = error {
        Ok(())
    } else {
        Err(error)
    }
}

fn map_invalid_position_to_ok<T>(error: Error<T>) -> Result<(), Error<T>> {
    use SetCursorError::*;

    if let Error::SetCursor(InvalidPosition) = error {
        Ok(())
    } else {
        Err(error)
    }
}

fn fix_offset_from_position(
    Offset { x, y }: Offset,
    position: Position,
    is_first_half: bool,
) -> Offset {
    let x = match position {
        Position { x: _, y: 2 } if is_first_half => 2 * x,
        Position {
            x: ..9 | 19..28,
            y: 2,
        } if x > 0 => 2 * x,
        Position {
            x: ..10 | 20..29,
            y: 2,
        } if x < 0 => 2 * x,
        Position { x: 29, y: _ } if x > 0 => 0,
        _ => x,
    };

    let y = match position {
        Position { x: _, y: 11 } if y > 0 => 0,
        _ => y,
    };

    Offset::new(x, y)
}
