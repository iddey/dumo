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

impl<'a, 'b, F, D, C> FlushWrapper<DumoBackend<'a, 'b, '_, '_, D, C>, F, D>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    #[cfg(feature = "alloc")]
    impl_with_blink!();
}
