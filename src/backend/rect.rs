use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::primitives::Rectangle;

/// Extension trait for rectangles.
pub trait RectangleExt {
    /// Returns the rectangle's area that only has pixels to the left of the specified area.
    fn left_of(&self, other: &Self) -> Self;

    /// Returns the rectangle's area that only has pixels to the right of the specified area.
    fn right_of(&self, other: &Self) -> Self;

    /// Returns the rectangle's area that only has pixels below the specified area.
    fn below(&self, other: &Self) -> Self;

    /// Returns the rectangle with its left side indented to the right, making the specified column
    /// its new left side.
    fn indent_to(&self, right: i32) -> Self;
}

impl RectangleExt for Rectangle {
    fn left_of(&self, other: &Self) -> Self {
        let top_left = self.top_left;
        let width = other.top_left.x.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, self.size.height);
        let size = self.size.component_min(size);

        Self { top_left, size }
    }

    fn right_of(&self, other: &Self) -> Self {
        let right = other.top_left.x.saturating_add_unsigned(other.size.width);
        let top_left = Point::new(right, self.top_left.y);
        let top_left = self.top_left.component_max(top_left);
        let width = right.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, Default::default());
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }

    fn below(&self, other: &Self) -> Self {
        let bottom = other.top_left.y.saturating_add_unsigned(other.size.height);
        let top_left = Point::new(self.top_left.x, bottom);
        let top_left = self.top_left.component_max(top_left);
        let height = bottom.saturating_sub(self.top_left.y);
        let height = height.try_into().unwrap_or_default();
        let size = Size::new(Default::default(), height);
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }

    fn indent_to(&self, right: i32) -> Self {
        let top_left = Point::new(right, self.top_left.y);
        let top_left = self.top_left.component_max(top_left);
        let width = right.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, Default::default());
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_left_of {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.left_of(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_left_of! {
        left_of_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(800, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(0, 4444)),

        left_of_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1600, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(1600 - 1111, 4444)),

        left_of_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(3200, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3200 - 1111, 4444)),

        left_of_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(6400, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        left_of_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        left_of_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),

        left_of_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(0, u32::MAX)),
    }

    macro_rules! test_right_of {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.right_of(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_right_of! {
        right_of_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(800, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        right_of_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(1600, 4444)),
            Rectangle::new(Point::new(1600, 2222), Size::new(3333 + 1111 - 1600, 4444)),

        right_of_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(3200, 4444)),
            Rectangle::new(Point::new(3200, 2222), Size::new(3333 + 1111 - 3200, 4444)),

        right_of_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(6400, 4444)),
            Rectangle::new(Point::new(6400, 2222), Size::new(0, 4444)),

        right_of_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        right_of_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX / 2 + 1, u32::MAX)),

        right_of_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
    }

    macro_rules! test_below {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.below(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_below! {
        below_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 800)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        below_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 1600)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        below_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 3200)),
            Rectangle::new(Point::new(1111, 3200), Size::new(3333, 4444 + 2222 - 3200)),

        below_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 6400)),
            Rectangle::new(Point::new(1111, 6400), Size::new(3333, 4444 + 2222 - 6400)),

        below_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        below_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX / 2)),
            Rectangle::new(Point::new(i32::MIN, -1), Size::new(u32::MAX, u32::MAX / 2 + 1)),

        below_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX / 2)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
    }

    macro_rules! test_indent_to {
        (
            $(
                $fn_ident:ident, $self:expr, $right:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.indent_to($right);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_indent_to! {
        indent_to_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            800,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        indent_to_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            1600,
            Rectangle::new(Point::new(1600, 2222), Size::new(3333 + 1111 - 1600, 4444)),

        indent_to_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            3200,
            Rectangle::new(Point::new(3200, 2222), Size::new(3333 + 1111 - 3200, 4444)),

        indent_to_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            6400,
            Rectangle::new(Point::new(6400, 2222), Size::new(0, 4444)),

        indent_to_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        indent_to_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            -1,
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX / 2 + 1, u32::MAX)),

        indent_to_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            -1,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
    }
}
