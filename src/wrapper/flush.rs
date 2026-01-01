use core::fmt::Debug;
use core::marker::PhantomData;

use embedded_graphics::draw_target::DrawTarget;
use ratatui_core::backend::{Backend, ClearType, WindowSize};
use ratatui_core::buffer::Cell;
use ratatui_core::layout::{Position, Size};

use crate::backend::DrawTargetBackend;
use crate::error::Error;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlushWrapper<B, F, D>
where
    B: DrawTargetBackend<F, D, Error = Error<D::Error>>,
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
    B: DrawTargetBackend<F, D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
{
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
    B: DrawTargetBackend<F, D, Error = Error<D::Error>>,
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
