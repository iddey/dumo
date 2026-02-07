use core::fmt::Debug;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use embedded_graphics::text::renderer::TextRenderer;
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use mplusfonts::style::BitmapFontStyle;

use crate::backend::DumoBackend;
use crate::blink::{Blink, ControlBlinking};
use crate::wrapper::blink::BlinkWrapper;
use crate::wrapper::cursor::CursorWrapper;
use crate::wrapper::flush::FlushWrapper;

impl_wrapper!(
    CursorWrapper['_],
    DumoBackend<'a, 'b, '_, '_, D, C>,
    impl_with_blink!(self, self.blinking(), self.stop_blinking());
    impl_with_flush!();
);

impl_wrapper!(
    CursorWrapper['_],
    BlinkWrapper<DumoBackend<'a, 'b, '_, '_, D, C>, D>,
    impl_with_flush!();
);

impl_wrapper!(
    CursorWrapper['_](F: FnMut(&mut D) -> Result<(), D::Error>),
    FlushWrapper<DumoBackend<'a, 'b, '_, '_, D, C>, F, D>,
    impl_with_blink!(self, self.blinking(), self.stop_blinking());
);
