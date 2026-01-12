use core::fmt::Debug;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use embedded_graphics::text::renderer::TextRenderer;
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use mplusfonts::style::BitmapFontStyle;

use crate::backend::DumoBackend;
use crate::wrapper::flush::FlushWrapper;

macro_rules! impl_with_flush {
    () => {
        /// Returns the backend with a new wrapper around it, adding the specified function item to
        /// the backend. Using this wrapper, the backend will proceed to call `flush_fn` on request
        /// from a [`Terminal`], when no further changes will be made to the draw target in a given
        /// frame; this allows for device drivers to be updated as part of `flush_fn` to push pixel
        /// information to the display.
        ///
        /// Backends without this wrapper take no action in the [`Backend::flush`] method.
        ///
        /// [`Backend::flush`]: ratatui_core::backend::Backend::flush
        /// [`Terminal`]: ratatui_core::terminal::Terminal
        pub const fn with_flush<F>(self, flush_fn: F) -> FlushWrapper<Self, F, D>
        where
            F: FnMut(&mut D) -> Result<(), D::Error>,
        {
            FlushWrapper::new(self, flush_fn)
        }
    };
}

impl<'a, 'b, D, C> DumoBackend<'a, 'b, '_, '_, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    impl_with_flush!();
}
