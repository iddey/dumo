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
use crate::blink::Blink;
#[cfg(feature = "alloc")]
use crate::wrapper::blink::BlinkWrapper;
use crate::wrapper::flush::FlushWrapper;

#[cfg(feature = "alloc")]
macro_rules! impl_with_blink {
    () => {
        /// Returns the backend with a new wrapper around it, redrawing cells to show and hide text
        /// that should blink. Every time a [`Terminal`] calls the [`Backend::draw`] method as part
        /// of the rendering process, the wrapper advances the blinking animation by one frame.
        ///
        /// Backends without this wrapper display text that is set to blink as solid text.
        ///
        /// [`Backend::draw`]: ratatui_core::backend::Backend::draw
        /// [`Terminal`]: ratatui_core::terminal::Terminal
        pub const fn with_blink(
            self,
            slow_blink: Blink,
            rapid_blink: Blink,
        ) -> BlinkWrapper<Self, D> {
            BlinkWrapper::new(self, slow_blink, rapid_blink)
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

macro_rules! impl_wrapper {
    (
        $wrapper_ident:ident
        $([$wrapper_lifetime:lifetime])?
        $(<$wrapper_type_param_ident:ident>)?
        $(($impl_type_param_ident:ident: $($impl_type_param_bounds:tt)+))?,
        $backend_type:ty
        $([$impl_lifetime:lifetime])?,
        $($impl_item:item)*
    ) => {
        impl<'a, 'b, $($impl_lifetime,)? $($impl_type_param_ident,)? D, C>
            $wrapper_ident<$($wrapper_lifetime,)? $backend_type, $($wrapper_type_param_ident,)? D>
        where
            C: PixelColor + From<C::Raw>,
            D: DrawTarget,
            D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
            D::Error: Debug,
            $($impl_type_param_ident: $($impl_type_param_bounds)+,)?
            RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
            BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
        {
            $($impl_item)*
        }
    }
}
