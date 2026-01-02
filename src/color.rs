//! Color palettes that allow the backend to render text in color.
//!
//! Color definitions are found in an instance of [`Palette`], while [`Palettes`] provides palettes
//! for quick configuration of the backend to use a color scheme other than the default one.

use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use ratatui_core::style::Color;

pub use crate::builder::PaletteBuilder;
pub use crate::palette::{Palette, Palettes};

/// Color mapping to type `T`, where an instance of [`Palette`] adds a layer of indirection between
/// the value to map and the return value.
///
/// A type that implements this trait has a set of mappings, one set for every [`Palette`] variant.
pub trait MapWith<T: PixelColor> {
    /// Maps the value, retrieving the corresponding color from the specified palette.
    fn map_with(&self, palette: Palette<T>) -> Option<T>;
}

impl<T: PixelColor + From<Rgb888>> MapWith<T> for Color {
    /// Maps the color, retrieving the corresponding color from the specified palette.
    ///
    /// # Examples
    ///
    /// ```
    /// use dumo::color::{MapWith, Palettes};
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics::prelude::*;
    /// # use ratatui::prelude::*;
    ///
    /// let color = Color::DarkGray;
    /// let palette = Rgb565::XTERM_256;
    /// let result = color.map_with(palette);
    ///
    /// assert_eq!(result, Some(Rgb565::CSS_GRAY));
    ///
    /// let color = Color::Reset;
    /// let palette = Rgb565::XTERM_256;
    /// let result = color.map_with(palette);
    ///
    /// assert_eq!(result, None);
    /// ```
    fn map_with(&self, palette: Palette<T>) -> Option<T> {
        match palette {
            Palette::Reset => None,
            Palette::Ansi16(colors) => match *self {
                Self::Reset => None,
                Self::Black => Some(colors[0]),
                Self::Red => Some(colors[1]),
                Self::Green => Some(colors[2]),
                Self::Yellow => Some(colors[3]),
                Self::Blue => Some(colors[4]),
                Self::Magenta => Some(colors[5]),
                Self::Cyan => Some(colors[6]),
                Self::Gray => Some(colors[7]),
                Self::DarkGray => Some(colors[8]),
                Self::LightRed => Some(colors[9]),
                Self::LightGreen => Some(colors[10]),
                Self::LightYellow => Some(colors[11]),
                Self::LightBlue => Some(colors[12]),
                Self::LightMagenta => Some(colors[13]),
                Self::LightCyan => Some(colors[14]),
                Self::White => Some(colors[15]),
                Self::Rgb(..96, ..96, ..96) => Some(colors[0]),
                Self::Rgb(96..176, ..96, ..96) => Some(colors[1]),
                Self::Rgb(..96, 96..176, ..96) => Some(colors[2]),
                Self::Rgb(96..176, 96..176, ..96) => Some(colors[3]),
                Self::Rgb(..96, ..96, 96..176) => Some(colors[4]),
                Self::Rgb(96..176, ..96, 96..176) => Some(colors[5]),
                Self::Rgb(..96, 96..176, 96..176) => Some(colors[6]),
                Self::Rgb(96..176, 96..176, 96..176) => Some(colors[8]),
                Self::Rgb(176.., ..176, ..176) => Some(colors[9]),
                Self::Rgb(..176, 176.., ..176) => Some(colors[10]),
                Self::Rgb(176.., 176.., ..176) => Some(colors[11]),
                Self::Rgb(..176, ..176, 176..) => Some(colors[12]),
                Self::Rgb(176.., ..176, 176..) => Some(colors[13]),
                Self::Rgb(..176, 176.., 176..) => Some(colors[14]),
                Self::Rgb(176..216, 176..216, 176..216) => Some(colors[7]),
                Self::Rgb(216.., 176..216, 176..216) => Some(colors[7]),
                Self::Rgb(176..216, 216.., 176..216) => Some(colors[7]),
                Self::Rgb(216.., 216.., 176..216) => Some(colors[15]),
                Self::Rgb(176..216, 176..216, 216..) => Some(colors[7]),
                Self::Rgb(216.., 176..216, 216..) => Some(colors[15]),
                Self::Rgb(176..216, 216.., 216..) => Some(colors[15]),
                Self::Rgb(216.., 216.., 216..) => Some(colors[15]),
                Self::Indexed(index @ 0..16) => Some(colors[index as usize]),
                Self::Indexed(16 | 17) => Some(colors[0]),
                Self::Indexed(18 | 19) => Some(colors[4]),
                Self::Indexed(20 | 21) => Some(colors[12]),
                Self::Indexed(22 | 28) => Some(colors[2]),
                Self::Indexed(23..28 | 29..34) => Some(colors[6]),
                Self::Indexed(34..52) => None,
                Self::Indexed(52 | 88) => Some(colors[1]),
                Self::Indexed(53..58 | 89..94) => Some(colors[5]),
                Self::Indexed(58 | 64 | 94 | 100) => Some(colors[3]),
                Self::Indexed(59..64 | 65..70 | 95..100 | 101..106) => Some(colors[8]),
                Self::Indexed(70..88 | 106..231) => None,
                Self::Indexed(231) => Some(colors[15]),
                Self::Indexed(232..240) => Some(colors[0]),
                Self::Indexed(240..248) => Some(colors[8]),
                Self::Indexed(248..252) => Some(colors[7]),
                Self::Indexed(252..) => Some(colors[15]),
            },
            Palette::Ansi256(colors) => match *self {
                Self::Reset => None,
                Self::Black => Some(colors[0]),
                Self::Red => Some(colors[1]),
                Self::Green => Some(colors[2]),
                Self::Yellow => Some(colors[3]),
                Self::Blue => Some(colors[4]),
                Self::Magenta => Some(colors[5]),
                Self::Cyan => Some(colors[6]),
                Self::Gray => Some(colors[7]),
                Self::DarkGray => Some(colors[8]),
                Self::LightRed => Some(colors[9]),
                Self::LightGreen => Some(colors[10]),
                Self::LightYellow => Some(colors[11]),
                Self::LightBlue => Some(colors[12]),
                Self::LightMagenta => Some(colors[13]),
                Self::LightCyan => Some(colors[14]),
                Self::White => Some(colors[15]),
                Self::Rgb(r, g, b) => Some(Rgb888::new(r, g, b).into()),
                Self::Indexed(index) => Some(colors[index as usize]),
            },
        }
    }
}
