//! [Ratatui](https://ratatui.rs) backend for use with [`embedded-graphics`](embedded_graphics);
//! this crate is compatible with `no_std` and renders [`mplusfonts`](../mplusfonts/index.html)
//! bitmap fonts to any [`DrawTarget`](embedded_graphics::draw_target::DrawTarget)-implementing
//! display device, so long as its associated pixel color type is one of those defined in the
//! [pixelcolor](embedded_graphics::pixelcolor) module, and its associated error type is [`Debug`].
//!
//! Support for device drivers that define their own color types — multicolor electrophoretic
//! displays — is implemented behind feature gates in [`mplusfonts`](../mplusfonts/index.html).
//! The prerequisite for adding such an implementation is that color types must implement the
//! [`Default`] and [`From<Rgb888>`] traits. As of `dumo` v0.1.1, such e-Paper display driver
//! crates, where the color types already implement all of the necessary traits, are limited
//! to [`epd-spectra`](https://crates.io/crates/epd-spectra).
//!
//! Otherwise, display drivers such as [`mipidsi`](https://crates.io/crates/mipidsi) do have
//! universal support as [`Rgb565`](embedded_graphics::pixelcolor::Rgb565) is defined in the
//! [pixelcolor](embedded_graphics::pixelcolor) module.

#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![doc = include_str!("../USAGE.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

mod backend;
mod builder;
mod palette;
mod wrapper;

pub mod blink;
pub mod color;
pub mod cursor;
pub mod error;
pub mod fonts;

pub use backend::*;

/// Creates a fixed-width bitmap font for a cell size of 6 by 16 pixels (**Wide**/**Small**).
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_6x16!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_6x16 {
    ($($args:tt)*) => {
        $crate::mpluscode!(115, 482, 16.125, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 6 by 18 pixels.
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_6x18!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_6x18 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 456, 18.06, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 8 by 20 pixels (**Wide**).
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_8x20!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_8x20 {
    ($($args:tt)*) => {
        $crate::mpluscode!(125, 444, 20.066667, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 8 by 24 pixels.
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_8x24!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_8x24 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 418, 24.08, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 8 by 24 pixels (**Bold**).
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_8x24_bold!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_8x24_bold {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 482, 24.08, true, $($args)*)

    }
}

/// Creates a fixed-width bitmap font for a cell size of 10 by 30 pixels.
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_10x30!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_10x30 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 454, 30.1, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 12 by 30 pixels (**Wide**).
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_12x30!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_12x30 {
    ($($args:tt)*) => {
        $crate::mpluscode!(125, 466, 30.1, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 12 by 36 pixels.
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_12x36!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_12x36 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 412, 36.12, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 12 by 36 pixels (**Bold**).
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_12x36_bold!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_12x36_bold {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 482, 36.12, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 14 by 42 pixels.
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_14x42!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_14x42 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 490, 42.14, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 16 by 40 pixels (**Wide**).
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_16x40!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_16x40 {
    ($($args:tt)*) => {
        $crate::mpluscode!(125, 500, 40.133333, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 16 by 48 pixels.
///
/// # Arguments
///
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::font_16x48!(4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! font_16x48 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 439, 48.16, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font.
///
/// # Arguments
///
/// * `width` - Font width. Ranges from `100` to `125`. `100` is `NORMAL`.
/// * `weight` - Font weight. Ranges from `100` to `700`. `400` is `NORMAL`.
/// * `height` - Line height in pixels. Rounded to an integer by the text shaper.
/// * `hint` - Set to `true` to enable font hinting and adjust text shapes to a pixel grid.
/// * `bit_depth` - Bit depth of glyph images. Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Zero or more sources of textual data. Ranges of characters and arrays of strings.
///
/// # Examples
///
/// ```
/// let bitmap_font = dumo::mpluscode!(125, 500, 40.133333, true, 4, '\x20'..'\x7F', ["‹›", "«»"]);
/// ```
#[macro_export]
macro_rules! mpluscode {
    ($width:tt, $weight:tt, $height:tt, $hint:tt, $($rest:tt)*) => {
        ::mplusfonts::mplus!(code($width), $weight, code_line_height($height), $hint, 1, $($rest)*)
    }
}
