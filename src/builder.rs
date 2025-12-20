use core::mem;
use core::mem::MaybeUninit;

use embedded_graphics::pixelcolor::PixelColor;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PaletteBuilder<T: PixelColor, const N: usize>([T; N]);

impl<T: PixelColor> PaletteBuilder<T, 0> {
    pub const fn new() -> Self {
        Self([])
    }
}

impl<T: PixelColor> PaletteBuilder<T, 256> {
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
