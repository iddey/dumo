use embedded_graphics::pixelcolor::raw::RawData;
use embedded_graphics::pixelcolor::*;
use seq_macro::seq;

use crate::color::PaletteBuilder;

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Palette<'a, T: PixelColor> {
    Reset,
    Ansi16(&'a [T; 16]),
    Ansi256(&'a [T; 256]),
}

pub trait Palettes<'a>: PixelColor + 'a {
    const XTERM_16: Palette<'a, Self>;

    const XTERM_256: Palette<'a, Self>;
}

trait GrayColorExt: GrayColor {
    const MAX_LUMA: u8 = 2u8
        .wrapping_pow(<Self as PixelColor>::Raw::BITS_PER_PIXEL as u32)
        .wrapping_sub(1);
}

impl<T: GrayColor> GrayColorExt for T {}

macro_rules! impl_palettes {
    ($color_type:ty, $r_ident:ident, $g_ident:ident, $b_ident:ident, $color:expr) => {
        impl<'a> Palettes<'a> for $color_type {
            const XTERM_16: Palette<'a, Self> = const {
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

            const XTERM_256: Palette<'a, Self> = const {
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
            impl_palettes!(
                $rgb_type,
                r,
                g,
                b,
                <$rgb_type>::new(
                    ceil(<$rgb_type>::MAX_R as f32 * r),
                    ceil(<$rgb_type>::MAX_G as f32 * g),
                    ceil(<$rgb_type>::MAX_B as f32 * b),
                )
            );
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
            impl_palettes!(
                $gray_type,
                r,
                g,
                b,
                <$gray_type>::new(
                    round(<$gray_type>::MAX_LUMA as f32 * luma(r, g, b))
                )
            );
        )*
    }
}

impl_palettes_gray!(Gray2, Gray4, Gray8);

impl_palettes!(
    BinaryColor,
    r,
    g,
    b,
    if luma(r, g, b) < 0.5 {
        BinaryColor::Off
    } else {
        BinaryColor::On
    }
);
