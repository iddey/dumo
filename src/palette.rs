use embedded_graphics::pixelcolor::raw::RawData;
use embedded_graphics::pixelcolor::*;
use seq_macro::seq;

use crate::color::PaletteBuilder;

/// Palette for looking up colors of type `T`, where `T` is the color associated with the
/// draw target for the backend.
///
/// Variants of this type do not own any arrays of colors. Some of the variants, however,
/// do hold references to arrays, which is the reason for having a lifetime parameter. To
/// define a custom palette instead of using one provided by the [`Palettes`] trait, make
/// an array of colors available, either in memory or as a `const`, for a palette to use.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Palette<'a, T: PixelColor> {
    /// A palette with no color information. Useful for monochrome displays because, with
    /// [`Palette::Reset`], all colors map to [`None`], discarding any color information.
    Reset,
    /// A palette with 16 color entries, with three color and one intensity channels, one
    /// bit per channel.
    ///
    /// With [`map_with`](crate::color::MapWith::map_with), there is a one-to-one mapping
    /// to the indices in the array of colors for
    /// [`Color::Black`](ratatui_core::style::Color::Black),
    /// [`Color::Red`](ratatui_core::style::Color::Red),
    /// etc.
    ///
    /// For every [`Color::Indexed`](ratatui_core::style::Color::Indexed) color, there is
    /// a mapping to one of the 16 color entries. The `colors` example demonstrates this.
    ///
    /// 24-bit [`Color::Rgb`](ratatui_core::style::Color::Rgb) colors are converted using
    /// pattern matching on the three color components, into one of the 16 color entries.
    Ansi16(&'a [T; 16]),
    /// A palette with 256 color entries:
    ///
    /// * `0..16` - The same 16 color entries that are found in [`Palette::Ansi16`].
    /// * `16..232` - The entries for a 6×6×6 color cube, which are standard colors.
    /// * `232..=255` - The entries for a grayscale ramp, excluding black and white.
    ///
    /// With [`map_with`](crate::color::MapWith::map_with), there is a one-to-one mapping
    /// to the indices in the array of colors for the indices in
    /// [`Color::Indexed`](ratatui_core::style::Color::Indexed) colors.
    ///
    /// 24-bit [`Color::Rgb`](ratatui_core::style::Color::Rgb) colors are converted using
    /// the [`embedded-graphics`](embedded_graphics) crate, calling `From<Rgb888>::from`.
    Ansi256(&'a [T; 256]),
}

/// Color palettes that are built in, ready to be used in the configuration of a backend.
///
/// A color that implements this trait has associated `const` items that provide palettes
/// with color definitions that are very similar to those that are often seen in terminal
/// emulation software.
pub trait Palettes<'a>: PixelColor + 'a {
    /// A palette that resembles the 16 colors of console applications running in Windows
    /// versions that do not support user-defined color schemes or have built-in presets.
    ///
    /// The red, green, and blue color channels are all sampled at 0, 50, and 100 percent
    /// levels, and non-bright white has levels set to 75 percent.
    const WIN_16: Palette<'a, Self>;

    /// Equivalent to [`Self::WIN_16`] for the first 16 colors, while the next 216 colors
    /// all have color channels that are sampled at 0, 20, 40, 60, 80, and 100 percent.
    ///
    /// The last 24 gray values range from around 4 percent to around 98 percent full.
    const WEB_256: Palette<'a, Self>;

    /// A palette that resembles the XTerm terminal emulator with its default 16 colors.
    ///
    /// Non-bright red, green, and blue have higher levels than in [`Self::WIN_16`].
    const XTERM_16: Palette<'a, Self>;

    /// A palette that resembles the XTerm terminal emulator with its default 256 colors.
    ///
    /// Non-bright red, green, and blue have higher levels than in [`Self::WIN_16`], with
    /// the next 216 colors, except for black, also shifted towards white. This gives the
    /// palette the impression of having a very basic gamma correction.
    ///
    /// The last 24 gray values range from around 3 percent to around 93 percent full.
    const XTERM_256: Palette<'a, Self>;
}

trait GrayColorExt: GrayColor {
    const MAX_LUMA: u8 = 2u8
        .wrapping_pow(Self::Raw::BITS_PER_PIXEL as u32)
        .wrapping_sub(1);
}

impl<T: GrayColor> GrayColorExt for T {}

macro_rules! impl_palettes {
    ($color_type:ty, $r_ident:ident, $g_ident:ident, $b_ident:ident, $color:expr) => {
        impl<'a> Palettes<'a> for $color_type {
            const WIN_16: Palette<'a, Self> = const {
                const fn color($r_ident: f32, $g_ident: f32, $b_ident: f32) -> $color_type {
                    $color
                }

                Palette::Ansi16(&[
                    color(0.0, 0.0, 0.0),
                    color(0.5, 0.0, 0.0),
                    color(0.0, 0.5, 0.0),
                    color(0.5, 0.5, 0.0),
                    color(0.0, 0.0, 0.5),
                    color(0.5, 0.0, 0.5),
                    color(0.0, 0.5, 0.5),
                    color(0.75, 0.75, 0.75),
                    color(0.5, 0.5, 0.5),
                    color(1.0, 0.0, 0.0),
                    color(0.0, 1.0, 0.0),
                    color(1.0, 1.0, 0.0),
                    color(0.0, 0.0, 1.0),
                    color(1.0, 0.0, 1.0),
                    color(0.0, 1.0, 1.0),
                    color(1.0, 1.0, 1.0),
                ])
            };

            const WEB_256: Palette<'a, Self> = const {
                const fn color($r_ident: f32, $g_ident: f32, $b_ident: f32) -> $color_type {
                    $color
                }

                const DATA: [$color_type; 256] = PaletteBuilder::new()
                    .head(&[
                        color(0.0, 0.0, 0.0),
                        color(0.5, 0.0, 0.0),
                        color(0.0, 0.5, 0.0),
                        color(0.5, 0.5, 0.0),
                        color(0.0, 0.0, 0.5),
                        color(0.5, 0.0, 0.5),
                        color(0.0, 0.5, 0.5),
                        color(0.75, 0.75, 0.75),
                        color(0.5, 0.5, 0.5),
                        color(1.0, 0.0, 0.0),
                        color(0.0, 1.0, 0.0),
                        color(1.0, 1.0, 0.0),
                        color(0.0, 0.0, 1.0),
                        color(1.0, 0.0, 1.0),
                        color(0.0, 1.0, 1.0),
                        color(1.0, 1.0, 1.0),
                    ])
                    .body(seq! {
                        N in 0..216 {
                            &[
                                #(
                                    {
                                        const R: u8 = N / 36;
                                        const G: u8 = N / 6 % 6;
                                        const B: u8 = N % 6;

                                        const fn value(index: u8) -> f32 {
                                            index as f32 / 5.0
                                        }

                                        color(value(R), value(G), value(B))
                                    },
                                )*
                            ]
                        }
                    })
                    .tail(seq! {
                        N in 0..24 {
                            &[
                                #(
                                    {
                                        let value = (N as f32 + 1.0) / 25.5;

                                        color(value, value, value)
                                    },
                                )*
                            ]
                        }
                    })
                    .build();

                Palette::Ansi256(&DATA)
            };

            const XTERM_16: Palette<'a, Self> = const {
                const fn color($r_ident: f32, $g_ident: f32, $b_ident: f32) -> $color_type {
                    $color
                }

                Palette::Ansi16(&[
                    color(0.0, 0.0, 0.0),
                    color(0.8, 0.0, 0.0),
                    color(0.0, 0.8, 0.0),
                    color(0.8, 0.8, 0.0),
                    color(0.0, 0.0, 14.0 / 15.0),
                    color(0.8, 0.0, 0.8),
                    color(0.0, 0.8, 0.8),
                    color(0.9, 0.9, 0.9),
                    color(0.5, 0.5, 0.5),
                    color(1.0, 0.0, 0.0),
                    color(0.0, 1.0, 0.0),
                    color(1.0, 1.0, 0.0),
                    color(23.0 / 64.0, 9.0 / 25.0, 1.0),
                    color(1.0, 0.0, 1.0),
                    color(0.0, 1.0, 1.0),
                    color(1.0, 1.0, 1.0),
                ])
            };

            const XTERM_256: Palette<'a, Self> = const {
                const fn color($r_ident: f32, $g_ident: f32, $b_ident: f32) -> $color_type {
                    $color
                }

                const DATA: [$color_type; 256] = PaletteBuilder::new()
                    .head(&[
                        color(0.0, 0.0, 0.0),
                        color(0.8, 0.0, 0.0),
                        color(0.0, 0.8, 0.0),
                        color(0.8, 0.8, 0.0),
                        color(0.0, 0.0, 14.0 / 15.0),
                        color(0.8, 0.0, 0.8),
                        color(0.0, 0.8, 0.8),
                        color(0.9, 0.9, 0.9),
                        color(0.5, 0.5, 0.5),
                        color(1.0, 0.0, 0.0),
                        color(0.0, 1.0, 0.0),
                        color(1.0, 1.0, 0.0),
                        color(23.0 / 64.0, 9.0 / 25.0, 1.0),
                        color(1.0, 0.0, 1.0),
                        color(0.0, 1.0, 1.0),
                        color(1.0, 1.0, 1.0),
                    ])
                    .body(seq! {
                        N in 0..216 {
                            &[
                                #(
                                    {
                                        const R: u8 = N / 36;
                                        const G: u8 = N / 6 % 6;
                                        const B: u8 = N % 6;

                                        const fn value(index: u8) -> f32 {
                                            if index > 0 {
                                                (5.0 * index as f32 + 7.0) / 32.0
                                            } else {
                                                0.0
                                            }
                                        }

                                        color(value(R), value(G), value(B))
                                    },
                                )*
                            ]
                        }
                    })
                    .tail(seq! {
                        N in 0..24 {
                            &[
                                #(
                                    {
                                        let value = (5.0 * N as f32 + 4.0) / 128.0;

                                        color(value, value, value)
                                    },
                                )*
                            ]
                        }
                    })
                    .build();

                Palette::Ansi256(&DATA)
            };
        }
    };
}

const fn ceil(value: f32) -> u8 {
    (value as u8)
        .checked_add(if value % 1.0 > 0.0 { 1 } else { 0 })
        .expect("expected value to be less than or equal to `255.0`")
}

macro_rules! impl_palettes_rgb {
    ($($rgb_type:ty),*) => {
        $(
            impl_palettes!($rgb_type, r, g, b, {
                <$rgb_type>::new(
                    ceil(<$rgb_type>::MAX_R as f32 * r),
                    ceil(<$rgb_type>::MAX_G as f32 * g),
                    ceil(<$rgb_type>::MAX_B as f32 * b),
                )
            });
        )*
    }
}

impl_palettes_rgb!(
    Rgb555, Bgr555, Rgb565, Bgr565, Rgb666, Bgr666, Rgb888, Bgr888
);

const fn luma(r: f32, g: f32, b: f32) -> f32 {
    // Recommendation ITU-R BT.709-2 luma coefficients.
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

const fn round(value: f32) -> u8 {
    (value as u8)
        .checked_add(if value % 1.0 < 0.5 { 0 } else { 1 })
        .expect("expected value to be less than `255.5`")
}

macro_rules! impl_palettes_gray {
    ($($gray_type:ty),*) => {
        $(
            impl_palettes!($gray_type, r, g, b, {
                <$gray_type>::new(
                    round(<$gray_type>::MAX_LUMA as f32 * luma(r, g, b))
                )
            });
        )*
    }
}

impl_palettes_gray!(Gray2, Gray4, Gray8);

impl_palettes!(BinaryColor, r, g, b, {
    if luma(r, g, b) < 0.5 {
        BinaryColor::Off
    } else {
        BinaryColor::On
    }
});

#[cfg(feature = "epd-spectra")]
impl_palettes!(epd_spectra::TriColor, r, g, b, {
    use epd_spectra::TriColor;

    // Adapt `impl From<Rgb888> for TriColor` for `f32`.
    let min = f32::min(f32::min(r, g), b);
    let max = f32::max(f32::max(r, g), b);
    let chroma = max - min;
    let brightness = max;

    // Function to match `impl From<Rgb888> for TriColor`.
    if chroma > 1.0 / 3.0 && r > g && r > b {
        TriColor::Red
    } else if brightness > 0.5 {
        TriColor::White
    } else {
        TriColor::Black
    }
});
