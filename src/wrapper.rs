#[macro_use]
mod wrap;

pub mod blink;
pub mod cursor;
pub mod flush;

use core::fmt::Debug;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::AnchorX;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use mplusfonts::BitmapFont;
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use ratatui_core::backend::Backend;
use ratatui_core::buffer::Cell;

use crate::backend::{ConfigureBackend, DrawTargetBackend};
use crate::color::Palette;
use crate::cursor::{Colors, Extent, Symbol};
use crate::error::Error;

/// Wrapper around an arbitrary object.
///
/// A backend wrapper that implements this trait is subject to blanket implementations that forward
/// function calls to a backend or another backend wrapper, given that the backend implements those
/// functions in the first place.
pub trait Wrapper {
    /// The type of the inner object.
    type Inner;

    /// Returns a reference to the inner object.
    fn inner(&self) -> &Self::Inner;

    /// Returns a reference with exclusive access to the inner object.
    fn inner_mut(&mut self) -> &mut Self::Inner;

    /// Consumes the wrapper, returning the inner object.
    fn into_inner(self) -> Self::Inner;
}

/// Marker for blanket implementations.
pub trait WrapTrait<T>: Wrapper {}

/// List of traits that are available to wrappers, where items are unit structs.
pub mod traits {
    #[derive(Debug, Clone, Copy)]
    pub struct DrawTargetBackend;
    #[derive(Debug, Clone, Copy)]
    pub struct ConfigureBackend;
    #[derive(Debug, Clone, Copy)]
    pub struct ConfigureBlinkWrapper;
    #[derive(Debug, Clone, Copy)]
    pub struct ConfigureCursorWrapper;
    #[derive(Debug, Clone, Copy)]
    pub struct ControlBlinking;
    #[derive(Debug, Clone, Copy)]
    pub struct ControlCursorBlinking;
}

/// Blanket implementation of the [`DrawTargetBackend`] trait for function call passthrough.
impl<W, B, D> DrawTargetBackend<D> for W
where
    B: DrawTargetBackend<D, Error = Error<D::Error>>,
    D: DrawTarget,
    D::Error: Debug,
    W: Backend<Error = B::Error>,
    W: WrapTrait<traits::DrawTargetBackend, Inner = B>,
{
    fn call(&mut self, f: impl FnMut(&mut D) -> Result<(), D::Error>) -> Result<(), D::Error> {
        self.inner_mut().call(f)
    }

    fn draw_hidden<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        self.inner_mut().draw_hidden(content)
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
        self.inner_mut()
            .draw_cursor(content, colors, extent, symbol)
    }

    fn advance_blink_by(&mut self, ticks: usize) -> Result<(), Self::Error> {
        self.inner_mut().advance_blink_by(ticks)
    }

    fn take_dirty_cursor(&mut self) -> Result<Option<()>, Self::Error> {
        self.inner_mut().take_dirty_cursor()
    }
}

/// Blanket implementation of the [`ConfigureBackend`] trait for function call passthrough.
impl<'a, 'b, 'c, W, B, T, C> ConfigureBackend<'a, 'b, 'c, T, C> for W
where
    B: ConfigureBackend<'a, 'b, 'c, T, C>,
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    W: WrapTrait<traits::ConfigureBackend, Inner = B>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    fn font(&self) -> &'b BitmapFont<'a, C, 1> {
        self.inner().font()
    }

    fn set_font(&mut self, font: &'b BitmapFont<'a, C, 1>) {
        self.inner_mut().set_font(font);
    }

    fn font_bold(&self) -> Option<&'b BitmapFont<'a, C, 1>> {
        self.inner().font_bold()
    }

    fn set_font_bold(&mut self, font_bold: Option<&'b BitmapFont<'a, C, 1>>) {
        self.inner_mut().set_font_bold(font_bold);
    }

    fn fg_reset(&self) -> Option<T> {
        self.inner().fg_reset()
    }

    fn set_fg_reset(&mut self, fg_reset: Option<T>) {
        self.inner_mut().set_fg_reset(fg_reset);
    }

    fn bg_reset(&self) -> Option<T> {
        self.inner().bg_reset()
    }

    fn set_bg_reset(&mut self, bg_reset: Option<T>) {
        self.inner_mut().set_bg_reset(bg_reset);
    }

    fn palette(&self) -> Palette<'c, T> {
        self.inner().palette()
    }

    fn set_palette(&mut self, palette: Palette<'c, T>) {
        self.inner_mut().set_palette(palette);
    }

    fn anchor_x(&self) -> AnchorX {
        self.inner().anchor_x()
    }

    fn set_anchor_x(&mut self, anchor_x: AnchorX) {
        self.inner_mut().set_anchor_x(anchor_x);
    }
}
