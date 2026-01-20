//! [Ratatui](https://ratatui.rs) backend for use with [`embedded-graphics`](embedded_graphics);
//! this crate is compatible with `no_std` and is still work-in-progress.

#![no_std]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod backend;
mod builder;
mod palette;
mod wrapper;

pub mod color;
pub mod error;

pub use backend::*;

/// Creates a fixed-width bitmap font for a cell size of 6 by 16 pixels (**Wide**/**Small**).
#[macro_export]
macro_rules! font_6x16 {
    ($($args:tt)*) => {
        $crate::mpluscode!(115, 482, 16.125, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 6 by 18 pixels.
#[macro_export]
macro_rules! font_6x18 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 456, 18.06, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 8 by 20 pixels (**Wide**).
#[macro_export]
macro_rules! font_8x20 {
    ($($args:tt)*) => {
        $crate::mpluscode!(125, 444, 20.066667, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 8 by 24 pixels.
#[macro_export]
macro_rules! font_8x24 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 418, 24.08, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 8 by 24 pixels (**Bold**).
#[macro_export]
macro_rules! font_8x24_bold {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 482, 24.08, true, $($args)*)

    }
}

/// Creates a fixed-width bitmap font for a cell size of 10 by 30 pixels.
#[macro_export]
macro_rules! font_10x30 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 454, 30.1, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 12 by 30 pixels (**Wide**).
#[macro_export]
macro_rules! font_12x30 {
    ($($args:tt)*) => {
        $crate::mpluscode!(125, 466, 30.1, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 12 by 36 pixels.
#[macro_export]
macro_rules! font_12x36 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 412, 36.12, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 12 by 36 pixels (**Bold**).
#[macro_export]
macro_rules! font_12x36_bold {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 482, 36.12, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 14 by 42 pixels.
#[macro_export]
macro_rules! font_14x42 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 490, 42.14, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 16 by 40 pixels (**Wide**).
#[macro_export]
macro_rules! font_16x40 {
    ($($args:tt)*) => {
        $crate::mpluscode!(125, 500, 40.133333, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font for a cell size of 16 by 48 pixels.
#[macro_export]
macro_rules! font_16x48 {
    ($($args:tt)*) => {
        $crate::mpluscode!(100, 439, 48.16, true, $($args)*)
    }
}

/// Creates a fixed-width bitmap font.
#[macro_export]
macro_rules! mpluscode {
    ($width:tt, $weight:tt, $height:tt, $hint:tt, $($rest:tt)*) => {
        ::mplusfonts::mplus!(code($width), $weight, code_line_height($height), $hint, 1, $($rest)*)
    }
}
