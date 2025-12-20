use embedded_graphics::geometry::Size;

/// Extension trait for size instances.
pub trait SizeExt: Sized {
    fn checked_component_mul(self, other: Self) -> Option<Self>;

    fn checked_component_div_ceil(self, other: Self) -> Option<Self>;

    fn checked_component_next_multiple_of(self, other: Self) -> Option<Self>;
}

impl SizeExt for Size {
    fn checked_component_mul(self, other: Self) -> Option<Self> {
        let width = self.width.checked_mul(other.width);
        let height = self.height.checked_mul(other.height);

        Some(Self::new(width?, height?))
    }

    fn checked_component_div_ceil(self, other: Self) -> Option<Self> {
        let width = (other.width > 0).then_some(self.width.div_ceil(other.width));
        let height = (other.height > 0).then_some(self.height.div_ceil(other.height));

        Some(Self::new(width?, height?))
    }

    fn checked_component_next_multiple_of(self, other: Self) -> Option<Self> {
        let width = self.width.checked_next_multiple_of(other.width);
        let height = self.height.checked_next_multiple_of(other.height);

        Some(Self::new(width?, height?))
    }
}
