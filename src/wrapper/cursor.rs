#![cfg(feature = "alloc")]

mod cache;
mod state;
mod wrap;

use core::fmt::Debug;
use core::marker::PhantomData;

use self::cache::CacheKey;
use self::state::State;
use embedded_graphics::draw_target::DrawTarget;
use ratatui_core::backend::{Backend, ClearType, WindowSize};
use ratatui_core::buffer::Cell;
use ratatui_core::layout::{Position, Size};
use ratatui_core::style::Modifier;

use crate::backend::DrawTargetBackend;
use crate::blink::{Blink, Blinked, ControlBlinking, ControlCursorBlinking};
use crate::cursor::{Colors, Cursor, Extent, Symbol};
use crate::error::{AdvanceCursorBlinkingError, Error};

use super::traits;
use super::{WrapTrait, Wrapper};

/// Wrapper that is required in order for a cursor to reveal itself whenever [`show_cursor`] or
/// [`set_cursor_position`] is called, keeping track of its position and driving its animation,
/// redrawing cells not only as the cursor is blinking, but also when the cursor leaves a cell.
///
/// For this purpose, the [`CursorWrapper`] layer maintains a cache with the last known content
/// of every cell position, removing entries on [`Backend::clear`] or [`Backend::clear_region`]
/// calls.
///
/// [`show_cursor`]: ratatui_core::terminal::Terminal::show_cursor
/// [`set_cursor_position`]: ratatui_core::terminal::Frame::set_cursor_position
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CursorWrapper<'a, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    backend: B,
    cursor: Cursor<'a>,
    state: State,
    phantom: PhantomData<D>,
}

/// Backend configuration retrieval and modification of the [`CursorWrapper`] layer.
///
/// A backend wrapper that implements this trait allows the fields of a [`CursorWrapper`] that are
/// configurable to have their values read or have new values assigned.
pub trait ConfigureCursorWrapper<'a> {
    /// Returns the blink animation cycle for the cursor.
    fn get_cursor_blink(&self) -> Blink;

    /// Sets the blink animation cycle for the cursor.
    fn set_cursor_blink(&mut self, blink: Blink);

    /// Returns the method used by the cursor for choosing colors.
    fn get_cursor_colors(&self) -> Colors;

    /// Sets the method used by the cursor for choosing colors.
    fn set_cursor_colors(&mut self, colors: Colors);

    /// Returns the shape that represents the cursor inside of the cell area.
    fn get_cursor_extent(&self) -> Extent;

    /// Sets the shape that represents the cursor inside of the cell area.
    fn set_cursor_extent(&mut self, extent: Extent);

    /// Returns the source of the content that the cursor uses to symbolize itself.
    fn get_cursor_symbol(&self) -> Symbol<'a>;

    /// Sets the source of the content that the cursor uses to symbolize itself.
    fn set_cursor_symbol(&mut self, symbol: Symbol<'a>);
}

impl<'a, B, D> CursorWrapper<'a, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    /// Creates a new wrapper around the specified backend, configuring the appearance of a cursor,
    /// indicating the last position that [`Backend::set_cursor_position`] receives as a parameter,
    /// first being positioned at the [`ORIGIN`](ratatui_core::layout::Position::ORIGIN).
    pub const fn new(backend: B, cursor: Cursor<'a>) -> Self {
        Self {
            backend,
            cursor,
            state: State::new(),
            phantom: PhantomData,
        }
    }

    fn draw_internal<'z, I, const HIDDEN: bool>(&mut self, content: I) -> Result<(), B::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        use unicode_width::UnicodeWidthStr;

        let cursor_blink = self.cursor.blink.get(self.state.ticks);
        let previous_cursor_blink = self.cursor.blink.get(self.state.ticks.wrapping_sub(1));
        let cursor_position = self.get_cursor_position()?;
        let mut cursor_content = None;

        let content = content.inspect(|&(x, y, cell)| {
            let key = CacheKey::new(x, y);
            let end = cell.symbol().width();
            let mut cell = cell.clone();

            if HIDDEN {
                cell.modifier = cell.modifier.union(Modifier::HIDDEN)
            }

            if y == cursor_position.y {
                for right in (0..end)
                    .filter_map(|x_offset| x_offset.try_into().ok())
                    .filter_map(|x_offset| x.checked_add(x_offset))
                {
                    if right == cursor_position.x {
                        cursor_content.replace((x, y, cell.clone()));
                    }
                }
            }

            self.state.cache.insert_or_replace(key, cell);
        });

        if HIDDEN {
            self.backend.draw_hidden(content)?;
        } else {
            self.backend.draw(content)?;
        }

        let cursor_is_visible = !self.state.cursor_hidden;
        let cursor_hidden_toggled = self.state.cursor_hidden_toggled.take().is_some();
        let cursor_blink_changed = cursor_blink != previous_cursor_blink;

        if let Some(position) = self.state.cursor_position_changed.take()
            && (cursor_is_visible || cursor_hidden_toggled)
            && (cursor_blink_changed || previous_cursor_blink == Blinked(true))
        {
            let cursor_content = self.state.cache.find(position);
            self.backend.draw(cursor_content.into_iter())?;
        }

        if cursor_is_visible {
            let cursor_content = match cursor_content.as_ref() {
                Some(&(x, y, ref cell)) => Some((x, y, cell)),
                None if cursor_blink_changed || self.state.cursor_changed => {
                    self.state.cache.find(cursor_position)
                }
                None => None,
            };

            if cursor_blink == Blinked(true) {
                self.backend.draw_cursor(
                    cursor_content.into_iter(),
                    self.cursor.colors,
                    self.cursor.extent,
                    self.cursor.symbol,
                )?;
            } else if cursor_blink_changed {
                self.backend.draw(cursor_content.into_iter())?;
            }
        }

        self.state.cursor_changed = false;

        if self.state.blinking {
            self.advance_blink_by(1)?;
        }

        Ok(())
    }
}

impl<B, D> Backend for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    type Error = B::Error;

    fn draw<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        self.draw_internal::<I, false>(content)
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        self.backend.hide_cursor()?;

        let changed = !self.state.cursor_hidden;
        if changed {
            self.state.cursor_hidden_toggled.get_or_insert(());
        }

        self.state.cursor_changed |= changed;
        self.state.cursor_hidden = true;

        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        self.backend.show_cursor()?;

        let changed = self.state.cursor_hidden;
        if changed {
            self.state.cursor_hidden_toggled.get_or_insert(());
        }

        self.state.cursor_changed |= changed;
        self.state.cursor_hidden = false;

        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        self.backend.get_cursor_position()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        let position = position.into();
        let cursor_position = self.get_cursor_position()?;

        self.backend.set_cursor_position(position)?;

        let changed = position != cursor_position;
        if changed {
            self.state
                .cursor_position_changed
                .get_or_insert(cursor_position);
        }

        self.state.cursor_changed |= changed;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.backend.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        self.backend.clear_region(clear_type)
    }

    fn size(&self) -> Result<Size, Self::Error> {
        self.backend.size()
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        self.backend.window_size()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.backend.flush()
    }
}

impl<B, D> DrawTargetBackend<D> for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    fn call(&mut self, f: impl FnMut(&mut D) -> Result<(), D::Error>) -> Result<(), D::Error> {
        self.backend.call(f)
    }

    fn draw_hidden<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        self.draw_internal::<I, true>(content)
    }

    fn draw_cursor<'z, I>(
        &mut self,
        content: I,
        colors: Colors,
        extent: Extent,
        symbol: Symbol,
    ) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        self.backend.draw_cursor(content, colors, extent, symbol)
    }

    fn advance_blink_by(&mut self, ticks: usize) -> Result<(), Self::Error> {
        self.state.ticks = self.state.ticks.wrapping_add(ticks);

        self.backend.advance_blink_by(ticks)
    }
}

impl<'a, B, D> ConfigureCursorWrapper<'a> for CursorWrapper<'a, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    fn get_cursor_blink(&self) -> Blink {
        self.cursor.blink
    }

    fn set_cursor_blink(&mut self, blink: Blink) {
        self.cursor.blink = blink;
    }

    fn get_cursor_colors(&self) -> Colors {
        self.cursor.colors
    }

    fn set_cursor_colors(&mut self, colors: Colors) {
        self.state.cursor_changed |= colors != self.cursor.colors;
        self.cursor.colors = colors;
    }

    fn get_cursor_extent(&self) -> Extent {
        self.cursor.extent
    }

    fn set_cursor_extent(&mut self, extent: Extent) {
        self.state.cursor_changed |= extent != self.cursor.extent;
        self.cursor.extent = extent;
    }

    fn get_cursor_symbol(&self) -> Symbol<'a> {
        self.cursor.symbol
    }

    fn set_cursor_symbol(&mut self, symbol: Symbol<'a>) {
        self.state.cursor_changed |= symbol != self.cursor.symbol;
        self.cursor.symbol = symbol;
    }
}

impl<B, D> ControlBlinking<D> for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    fn blinking(&self) -> bool {
        self.state.blinking
    }

    fn start_blinking(&mut self) {
        self.state.blinking = true;
    }

    fn stop_blinking(&mut self) {
        self.state.blinking = false;
    }
}

impl<B, D> ControlCursorBlinking for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    fn advance_cursor_blink_to(&mut self, blinked: Blinked) -> Result<(), B::Error> {
        use AdvanceCursorBlinkingError::*;

        self.state.ticks = match self.cursor.blink {
            Blink::Repeat(delay, blink) => match blinked {
                Blinked(false) => {
                    if delay == 0 {
                        Err(Error::AdvanceCursorBlinking(InvalidBlinked))
                    } else if let Some(period) = delay.checked_add(blink) {
                        let cycles = self.state.ticks.div_ceil(period);
                        let ticks = cycles.wrapping_mul(period);

                        Ok(ticks)
                    } else {
                        Ok(0)
                    }
                }
                Blinked(true) => {
                    if blink == 0 {
                        Err(Error::AdvanceCursorBlinking(InvalidBlinked))
                    } else if let Some(period) = delay.checked_add(blink) {
                        let cycles = self.state.ticks.saturating_sub(delay).div_ceil(period);
                        let ticks = cycles.wrapping_mul(period).saturating_add(delay);

                        Ok(ticks)
                    } else {
                        Ok(delay)
                    }
                }
            },
        }?;

        Ok(())
    }
}

impl<B, D> Wrapper for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    type Inner = B;

    fn inner(&self) -> &Self::Inner {
        &self.backend
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.backend
    }

    fn into_inner(self) -> Self::Inner {
        self.backend
    }
}

impl<B, D> WrapTrait<traits::ConfigureBackend> for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
}

impl<B, D> WrapTrait<traits::ConfigureBlinkWrapper> for CursorWrapper<'_, B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
}

/// Blanket implementation of the [`ConfigureCursorWrapper`] trait for function call passthrough.
impl<'a, W, B> ConfigureCursorWrapper<'a> for W
where
    B: ConfigureCursorWrapper<'a>,
    W: WrapTrait<traits::ConfigureCursorWrapper, Inner = B>,
{
    fn get_cursor_blink(&self) -> Blink {
        self.inner().get_cursor_blink()
    }

    fn set_cursor_blink(&mut self, blink: Blink) {
        self.inner_mut().set_cursor_blink(blink);
    }

    fn get_cursor_colors(&self) -> Colors {
        self.inner().get_cursor_colors()
    }

    fn set_cursor_colors(&mut self, colors: Colors) {
        self.inner_mut().set_cursor_colors(colors);
    }

    fn get_cursor_extent(&self) -> Extent {
        self.inner().get_cursor_extent()
    }

    fn set_cursor_extent(&mut self, extent: Extent) {
        self.inner_mut().set_cursor_extent(extent);
    }

    fn get_cursor_symbol(&self) -> Symbol<'a> {
        self.inner().get_cursor_symbol()
    }

    fn set_cursor_symbol(&mut self, symbol: Symbol<'a>) {
        self.inner_mut().set_cursor_symbol(symbol);
    }
}
