//! Types for specifying how to render a cursor.

use embedded_graphics::primitives::Rectangle;
use ratatui_core::style::Color;

use crate::blink::Blink;

/// Method of choosing colors for a cursor.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Colors {
    /// Render the cursor with [`DumoBackend::fg_reset`](crate::backend::DumoBackend::fg_reset) and
    /// [`DumoBackend::bg_reset`](crate::DumoBackend::bg_reset) swapped with each other to become
    /// the foreground and background colors.
    ReversedReset,
    /// Render the cursor with [`DumoBackend::fg_reset`](crate::backend::DumoBackend::fg_reset) and
    /// [`DumoBackend::bg_reset`](crate::DumoBackend::bg_reset) negated by calling the [`invert`]
    /// method on both to become the foreground and background colors.
    ///
    /// [`invert`]: mplusfonts::color::Invert::invert
    InvertedReset,
    /// Render the cursor with custom foreground and background colors.
    Custom {
        /// The foreground color of the cursor.
        fg: Color,
        /// The background color of the cursor.
        bg: Color,
    },
}

/// Shape that represents the cursor inside of the cell area.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Extent {
    /// Render the cursor with a rectangular shape that fills the entire area of the cell or cells.
    FullBlock,
    /// Render the cursor with a rectangular shape that has the specified width and is positioned
    /// inside of the cell area, against its left side, extending to the right while also filling
    /// the entire height of the cell.
    VerticalBar {
        /// The width of the vertical bar.
        width: u32,
    },
    /// Render the cursor with a rectangular shape that has the specified height and is positioned
    /// below the text baseline, where the top of the text underline is also positioned, extending
    /// downwards and filling the entire width of the cell or cells.
    Underline {
        /// The height of the underline.
        height: u32,
    },
    /// Render the cursor with a custom rectangular shape that is positioned inside of the cell
    /// area in the top-left corner, offset by [`Rectangle::top_left`] and having the specified
    /// [`Rectangle::size`].
    Custom(Rectangle),
}

/// Source of the content that a cursor uses to symbolize itself.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Symbol<'a> {
    /// Render the cursor with the symbol found at the position of the cursor, using the foreground
    /// and background colors assigned to the cursor.
    UnderCursor,
    /// Render the cursor with a custom symbol (for example, `🮕`) that is found in the bitmap font,
    /// specified as a string slice.
    Custom(&'a str),
}

/// Cursor indicator appearance settings.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Cursor<'a> {
    /// The blink animation cycle.
    pub blink: Blink,
    /// The method of choosing colors.
    pub colors: Colors,
    /// The extent of the drawing area.
    pub extent: Extent,
    /// The source of the symbol drawn.
    pub symbol: Symbol<'a>,
}

impl Symbol<'_> {
    /// The cursor shape is filled with a solid color, the background color assigned to the cursor,
    /// or the foreground color assigned when [`REVERSED`](ratatui_core::style::Modifier::REVERSED)
    /// text is found at the position of the cursor.
    pub const SPACE: Self = Self::Custom(" ");
}

impl Cursor<'_> {
    /// Creates a new cursor that is solid on, uses the [`Reset`](Color::Reset) colors swapped with
    /// one another, drawing to the entire area of the cell or cells. This is default configuration
    /// for a cursor, and it also renders the symbol found at the position of the cursor.
    pub const fn new() -> Self {
        Self {
            blink: Blink::with_period(0),
            colors: Colors::ReversedReset,
            extent: Extent::FullBlock,
            symbol: Symbol::UnderCursor,
        }
    }

    /// Sets the blinking animation to have a single frame that always shows the cursor being on.
    #[must_use = "method consumes the cursor and returns one with the new settings"]
    pub const fn solid_on(self) -> Self {
        self.with_blink(Blink::with_period(0))
    }

    /// Sets the foreground and background colors to the same one that always shows the cursor as
    /// having a single color.
    #[must_use = "method consumes the cursor and returns one with the new settings"]
    pub const fn solid_color(self, color: Color) -> Self {
        self.with_colors(Colors::Custom {
            fg: color,
            bg: color,
        })
    }

    /// Sets the blink animation cycle for the cursor.
    #[must_use = "method consumes the cursor and returns one with the new settings"]
    pub const fn with_blink(mut self, blink: Blink) -> Self {
        self.blink = blink;
        self
    }

    /// Sets the method used by the cursor for choosing colors.
    #[must_use = "method consumes the cursor and returns one with the new settings"]
    pub const fn with_colors(mut self, colors: Colors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the shape that represents the cursor inside of the cell area.
    #[must_use = "method consumes the cursor and returns one with the new settings"]
    pub const fn with_extent(mut self, extent: Extent) -> Self {
        self.extent = extent;
        self
    }

    /// Sets the source of the content that the cursor uses to symbolize itself.
    #[must_use = "method consumes the cursor and returns one with the new settings"]
    pub const fn with_symbol<'a>(self, symbol: Symbol<'a>) -> Cursor<'a> {
        Cursor { symbol, ..self }
    }
}

impl Default for Cursor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Colors {
    fn format(&self, formatter: defmt::Formatter) {
        match self {
            Self::InvertedReset => defmt::write!(formatter, "InvertedReset"),
            Self::ReversedReset => defmt::write!(formatter, "ReversedReset"),
            Self::Custom { fg, bg } => {
                defmt::write!(
                    formatter,
                    "Custom {{ fg: {:?}, bg: {:?} }}",
                    defmt::Debug2Format(fg),
                    defmt::Debug2Format(bg)
                )
            }
        }
    }
}
