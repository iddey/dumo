use core::fmt::Debug;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use embedded_graphics::text::renderer::TextRenderer;
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use mplusfonts::style::BitmapFontStyle;

use crate::backend::DumoBackend;
#[cfg(feature = "alloc")]
use crate::wrapper::blink::BlinkWrapper;
use crate::wrapper::flush::FlushWrapper;

#[cfg(feature = "alloc")]
macro_rules! impl_with_blink {
    () => {
        /// Returns the backend with a new wrapper around it, redrawing cells to show and hide text
        /// that should blink. Every time a [`Terminal`] calls the [`Backend::draw`] method as part
        /// of the rendering process, the wrapper advances the blinking animation by one frame. The
        /// `long_period` and `short_period` parameters apply to [`SLOW_BLINK`] and [`RAPID_BLINK`]
        /// modifiers, respectively, where the number of frames in an animation cycle is specified.
        ///
        /// Backends without this wrapper display text that is set to blink as solid text.
        ///
        /// [`Backend::draw`]: ratatui_core::backend::Backend::draw
        /// [`Terminal`]: ratatui_core::terminal::Terminal
        /// [`SLOW_BLINK`]: ratatui_core::style::Modifier::SLOW_BLINK
        /// [`RAPID_BLINK`]: ratatui_core::style::Modifier::RAPID_BLINK
        pub const fn with_blink(
            self,
            long_period: usize,
            short_period: usize,
        ) -> BlinkWrapper<Self, D> {
            BlinkWrapper::new(self, long_period, short_period)
        }
    };
}

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
    #[cfg(feature = "alloc")]
    impl_with_blink!();
    impl_with_flush!();
}
