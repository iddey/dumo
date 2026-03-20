use core::mem;
use core::mem::MaybeUninit;

use embedded_graphics::pixelcolor::PixelColor;

/// Builder for a palette for looking up colors of type `T`, where `T` is the color associated with
/// the draw target for the backend.
///
/// The methods of this type provide a fluent-style interface for building 256-color lookup tables,
/// which are intended to be used for creating [`Ansi256`](crate::color::Palette::Ansi256) palettes
/// without having to keep track of whether a given subset of colors have been initialized.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PaletteBuilder<T: PixelColor, const N: usize>([T; N]);

impl<T: PixelColor> PaletteBuilder<T, 0> {
    /// Creates a new, empty builder with an empty array of colors having type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dumo::color::PaletteBuilder;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// let builder: PaletteBuilder<Rgb565, _> = PaletteBuilder::new();
    /// ```
    pub const fn new() -> Self {
        Self([])
    }
}

impl<T: PixelColor> PaletteBuilder<T, 256> {
    /// Consumes the builder, returning the array of colors having type `T`.
    ///
    /// This method is only available after all 256 colors have been added.
    ///
    /// # Examples
    ///
    /// ```
    /// use dumo::color::PaletteBuilder;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// let palette = PaletteBuilder::new()
    ///     .head(&[Rgb565::default(); 16])
    ///     .body(&[Rgb565::default(); 216])
    ///     .tail(&[Rgb565::default(); 24])
    ///     .build();
    /// ```
    pub const fn build(self) -> [T; 256] {
        self.0
    }
}

macro_rules! impl_add {
    (
        $(
            $fn_ident:ident, $array_length:literal, $mid:expr, $n:literal,
        )*
    ) => {
        $(
            impl<T: PixelColor> PaletteBuilder<T, $array_length> {
                /// Adds a number of colors, inserting them at the expected position in a new array
                /// where the colors that have previously been added are also moved.
                ///
                /// # Examples
                ///
                /// ```
                /// use dumo::color::PaletteBuilder;
                /// # use embedded_graphics::pixelcolor::Rgb565;
                ///
                #[doc = concat!(
                    "let palette = PaletteBuilder::new().",
                    stringify!($fn_ident),
                    "(&[Rgb565::default(); ",
                    stringify!($n - $array_length),
                    "]);",
                )]
                /// ```
                #[must_use = "method copies new and any existing colors to a new array"]
                pub const fn $fn_ident(
                    self,
                    colors: &[T; $n - $array_length]
                ) -> PaletteBuilder<T, $n> {
                    let Self(array) = self;
                    let mut new_array: [MaybeUninit<T>; $n] = [MaybeUninit::uninit(); _];

                    let (src_left, src_right) = array.split_at($mid);
                    let (dst_left, dst_right) = new_array.split_at_mut($mid);
                    let (dst_middle, dst_right) = dst_right.split_at_mut(colors.len());

                    // SAFETY: `&[T]` and `&[MaybeUninit<T>]` have the same layout.
                    let a = unsafe { mem::transmute::<&[T], &[MaybeUninit<T>]>(src_left) };
                    let b = unsafe { mem::transmute::<&[T], &[MaybeUninit<T>]>(colors.as_slice()) };
                    let c = unsafe { mem::transmute::<&[T], &[MaybeUninit<T>]>(src_right) };

                    dst_left.copy_from_slice(a);
                    dst_middle.copy_from_slice(b);
                    dst_right.copy_from_slice(c);

                    let new_array = new_array.as_ptr().cast::<[_; _]>();

                    // SAFETY: The full set of colors will have been initialized by this point.
                    PaletteBuilder(unsafe { new_array.read() })
                }
            }
        )*
    }
}

impl_add! {
    head, 0, 0, 16,
    head, 24, 0, 40,
    head, 216, 0, 232,
    head, 240, 0, 256,

    body, 0, 0, 216,
    body, 16, 16, 232,
    body, 24, 0, 240,
    body, 40, 16, 256,

    tail, 0, 0, 24,
    tail, 16, 16, 40,
    tail, 216, 216, 240,
    tail, 232, 232, 256,
}

impl<T: PixelColor> Default for PaletteBuilder<T, 0> {
    fn default() -> Self {
        Self::new()
    }
}
