#![cfg(feature = "alloc")]

mod cache;
mod state;
mod wrap;

use core::fmt::Debug;
use core::iter;
use core::marker::PhantomData;

use self::cache::CacheKey;
use self::state::State;
use embedded_graphics::draw_target::DrawTarget;
use ratatui_core::backend::{Backend, ClearType, WindowSize};
use ratatui_core::buffer::Cell;
use ratatui_core::layout::{Position, Size};
use ratatui_core::style::Modifier;

use crate::backend::DrawTargetBackend;
use crate::error::Error;

use super::Wrapper;

/// Blinked state representation. If the inner value is `true`, then text should be hidden, `false`
/// means that text is visible and drawn to the target because it has not _blinked_.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Blinked(bool);

/// Blink animation definition. At the moment, blinking is always a 50%–50% split between frames of
/// text being visible and hidden, where the odd frame is added to the delay before being blinking.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Blink {
    /// A period split into two parts, the first part is a delay before having _blinked_, while the
    /// second one is the remainder of the period, when text has _blinked_.
    Delayed(usize, usize),
}

/// Wrapper that is required in order for text that has [`slow_blink`] or [`rapid_blink`] modifiers
/// added to its style, to appear to be blinking, driving the animation by redrawing cells that are
/// stored in a cache until new content with intersecting positions and without blinking are drawn.
///
/// [`slow_blink`]: ratatui_core::style::Stylize::slow_blink
/// [`rapid_blink`]: ratatui_core::style::Stylize::rapid_blink
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlinkWrapper<B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    backend: B,
    slow_blink: Blink,
    rapid_blink: Blink,
    state: State,
    phantom: PhantomData<D>,
}

impl Blink {
    /// Creates a new blink animation cycle with the specified period, where text is first visible,
    /// then hidden for a duration that is half the period, possibly one frame less than the first.
    pub const fn from_period(period: usize) -> Self {
        let blink = period / 2;
        let delay = period - blink;

        Self::Delayed(delay, blink)
    }

    /// Returns whether text has _blinked_ in a given frame, specified with a zero-based index from
    /// the beginning of the animation cycle, wrapping if greater than or equal to the period.
    pub const fn get(&self, index: usize) -> Blinked {
        match *self {
            Self::Delayed(delay, blink) => {
                if let Some(index) = index.checked_rem(delay + blink) {
                    Blinked(index >= delay)
                } else {
                    Blinked(true)
                }
            }
        }
    }
}

impl<B, D> BlinkWrapper<B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    /// Creates a new wrapper around the specified backend, configuring the slow and rapid blinking
    /// animation cycles to have the specified periods, long and short, in frames.
    pub const fn new(backend: B, long_period: usize, short_period: usize) -> Self {
        Self {
            backend,
            slow_blink: Blink::from_period(long_period),
            rapid_blink: Blink::from_period(short_period),
            state: State::new(),
            phantom: PhantomData,
        }
    }
}

impl<B, D> Backend for BlinkWrapper<B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    type Error = B::Error;

    fn draw<'z, I>(&mut self, mut content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        const SPARSE_BLINK: Modifier = Modifier::SLOW_BLINK.union(Modifier::RAPID_BLINK);
        const ALL_BLINK: Modifier = SPARSE_BLINK;

        let slow_blink = self.slow_blink.get(self.state.ticks);
        let rapid_blink = self.rapid_blink.get(self.state.ticks);
        let sparse_blink = Blinked(slow_blink.0 && rapid_blink.0);

        let previous_slow_blink = self.slow_blink.get(self.state.ticks.wrapping_sub(1));
        let previous_rapid_blink = self.rapid_blink.get(self.state.ticks.wrapping_sub(1));
        let previous_sparse_blink = Blinked(previous_slow_blink.0 && previous_rapid_blink.0);

        let content = iter::from_fn(|| {
            if let Some((x, y, cell)) = content.next() {
                let key = CacheKey::new(x, y);
                if cell.modifier.intersects(ALL_BLINK) {
                    self.state.cache.insert_or_replace(key, cell.clone());
                } else {
                    self.state.cache.remove(&key);
                }

                Some((x, y, cell))
            } else {
                self.state.ticks = self.state.ticks.wrapping_add(1);

                None
            }
        });

        let no_blink_content = content.filter(|(_, _, cell)| !cell.modifier.intersects(ALL_BLINK));

        self.backend.draw(no_blink_content)?;

        for (blink, previous_blink, blink_flags) in [
            (slow_blink, previous_slow_blink, Modifier::SLOW_BLINK),
            (rapid_blink, previous_rapid_blink, Modifier::RAPID_BLINK),
            (sparse_blink, previous_sparse_blink, SPARSE_BLINK),
        ] {
            let blink_changed = blink != previous_blink;
            let blink_content = self.state.cache.iter().filter_map(|item| {
                let has_flags = blink_flags == item.cell.modifier.intersection(ALL_BLINK);
                let has_changed = blink_changed || item.changed;

                (has_flags && has_changed).then_some((item.key.x, item.key.y, &item.cell))
            });

            if blink == Blinked(true) {
                self.backend.draw_hidden(blink_content)?;
            } else {
                self.backend.draw(blink_content)?;
            }
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        self.backend.hide_cursor()
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        self.backend.show_cursor()
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        self.backend.get_cursor_position()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        self.backend.set_cursor_position(position)
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

impl<B, D> Wrapper for BlinkWrapper<B, D>
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
