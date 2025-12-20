use embedded_graphics::geometry::Size;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::BigEndian;
use mplusfonts::BitmapFont;

pub trait CellSize {
    fn cell_size(&self) -> Size;
}

impl<'a, C, const N: usize> CellSize for BitmapFont<'a, C, N>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    fn cell_size(&self) -> Size {
        Size {
            width: (self.charmap.get("\u{FFFD}").advance_width_to)("\u{FFFD}") as u32,
            height: self.metrics.line_height(),
        }
    }
}
