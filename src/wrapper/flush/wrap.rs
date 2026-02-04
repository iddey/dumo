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

impl_wrapper!(
    FlushWrapper<F>(F: FnMut(&mut D) -> Result<(), D::Error>),
    DumoBackend<'a, 'b, '_, '_, D, C>,
    #[cfg(feature = "alloc")]
    impl_with_blink!();
);
