mod wrap;

use core::fmt::Debug;
use core::marker::PhantomData;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use ratatui_core::backend::{Backend, ClearType, WindowSize};
use ratatui_core::buffer::Cell;
use ratatui_core::layout::{Position, Size};

use crate::backend::{ConfigureBackend, DrawTargetBackend};
use crate::error::Error;
use crate::wrapper::{ConfigureBackendWrapper, DrawTargetBackendWrapper};

use super::Wrapper;

/// Wrapper with a function item for backends to call, flushing the changes made to the draw target
/// on request, which is needed for certain display device drivers.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
{
    backend: B,
    flush_fn: F,
    phantom: PhantomData<D>,
}

impl<B, F, D> FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
{
    /// Creates a new wrapper around the specified backend, configuring the specified function item
    /// as the one to invoke when the flush method of the backend is called. All other methods call
    /// the backend as if there was no wrapper around it.
    pub const fn new(backend: B, flush_fn: F) -> Self {
        Self {
            backend,
            flush_fn,
            phantom: PhantomData,
        }
    }
}

impl<B, F, D> Backend for FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
{
    type Error = B::Error;

    fn draw<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        self.backend.draw(content)
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
        self.backend.call(&mut self.flush_fn).map_err(Error::Flush)
    }
}

impl<B, F, D> Wrapper for FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
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

impl<B, F, D> DrawTargetBackendWrapper<B, D> for FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
{
}

impl<'a, 'b, 'c, B, F, D, C> ConfigureBackendWrapper<'a, 'b, 'c, B, D::Color, C>
    for FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<D, Error = Error<D::Error>> + ConfigureBackend<'a, 'b, 'c, D::Color, C>,
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
}
