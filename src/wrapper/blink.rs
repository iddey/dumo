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
use crate::blink::{Blink, Blinked, ControlBlinking};
use crate::error::Error;

use super::traits;
use super::{WrapTrait, Wrapper};

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

/// Backend configuration retrieval and modification of the [`BlinkWrapper`] layer.
///
/// A backend wrapper that implements this trait allows the fields of a [`BlinkWrapper`] that are
/// configurable to have their values read or have new values assigned.
pub trait ConfigureBlinkWrapper {
    /// Returns the blink animation cycle for slow blinking.
    fn slow_blink(&self) -> Blink;

    /// Sets the blink animation cycle for slow blinking.
    fn set_slow_blink(&mut self, slow_blink: Blink);

    /// Returns the blink animation cycle for rapid blinking.
    fn rapid_blink(&self) -> Blink;

    /// Sets the blink animation cycle for rapid blinking.
    fn set_rapid_blink(&mut self, rapid_blink: Blink);
}

impl<B, D> BlinkWrapper<B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    /// Creates a new wrapper around the specified backend, configuring the slow and rapid blinking
    /// animation cycles.
    pub const fn new(backend: B, slow_blink: Blink, rapid_blink: Blink) -> Self {
        Self {
            backend,
            slow_blink,
            rapid_blink,
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

        if self.state.blinking {
            self.advance_blink_by(1)?;
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

impl<B, D> DrawTargetBackend<D> for BlinkWrapper<B, D>
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
        self.backend.draw_hidden(content)
    }

    fn advance_blink_by(&mut self, ticks: usize) -> Result<(), Self::Error> {
        self.state.ticks = self.state.ticks.wrapping_add(ticks);

        self.backend.advance_blink_by(ticks)
    }
}

impl<B, D> ConfigureBlinkWrapper for BlinkWrapper<B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
    fn slow_blink(&self) -> Blink {
        self.slow_blink
    }

    fn set_slow_blink(&mut self, slow_blink: Blink) {
        self.slow_blink = slow_blink;
    }

    fn rapid_blink(&self) -> Blink {
        self.rapid_blink
    }

    fn set_rapid_blink(&mut self, rapid_blink: Blink) {
        self.rapid_blink = rapid_blink;
    }
}

impl<B, D> ControlBlinking<D> for BlinkWrapper<B, D>
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

impl<B, D> WrapTrait<traits::ConfigureBackend> for BlinkWrapper<B, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
{
}

impl<W, B> ConfigureBlinkWrapper for W
where
    B: ConfigureBlinkWrapper,
    W: WrapTrait<traits::ConfigureBlinkWrapper, Inner = B>,
{
    fn slow_blink(&self) -> Blink {
        self.inner().slow_blink()
    }

    fn set_slow_blink(&mut self, slow_blink: Blink) {
        self.inner_mut().set_slow_blink(slow_blink);
    }

    fn rapid_blink(&self) -> Blink {
        self.inner().rapid_blink()
    }

    fn set_rapid_blink(&mut self, rapid_blink: Blink) {
        self.inner_mut().set_rapid_blink(rapid_blink);
    }
}
