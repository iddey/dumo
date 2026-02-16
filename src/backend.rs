mod cell;
mod rect;
mod size;
mod state;

use core::fmt::Debug;

use self::cell::CellSize;
use self::rect::RectangleExt;
use self::size::SizeExt;
use self::state::State;
use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::{AnchorX, Point};
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, DecorationColor};
use embedded_graphics::transform::Transform;
use mplusfonts::BitmapFont;
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use mplusfonts::style::BitmapFontStyle;
use ratatui_core::backend::{Backend, ClearType, WindowSize};
use ratatui_core::buffer::Cell;
use ratatui_core::layout::{Position, Size};
use ratatui_core::style::Modifier;

use crate::color::{MapWith, Palette, Palettes};
use crate::cursor::{Colors, Extent, Symbol};
use crate::error::{Error, MeasureError, SetCursorError};

pub use crate::wrapper::Wrapper;
#[cfg(feature = "alloc")]
pub use crate::wrapper::blink::{BlinkWrapper, ConfigureBlinkWrapper};
#[cfg(feature = "alloc")]
pub use crate::wrapper::cursor::{ConfigureCursorWrapper, CursorWrapper};
pub use crate::wrapper::flush::FlushWrapper;

/// Backend for Ratatui that renders to a display with the [`embedded-graphics`](embedded_graphics)
/// crate, using fixed-width bitmap fonts from the [`mplusfonts`] crate.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DumoBackend<'a, 'b, 'c, 'd, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    /// The draw target.
    pub target: &'d mut D,
    /// The bitmap font to use in general.
    pub font: &'b BitmapFont<'a, C, 1>,
    /// The bitmap font for when text should be bold. Defaults to the regular font.
    pub font_bold: Option<&'b BitmapFont<'a, C, 1>>,
    /// The foreground color to use in case no specific color is set. Defaults to white or enabled.
    pub fg_reset: Option<D::Color>,
    /// The background color to use in case no specific color is set. Defaults to black or disabled.
    pub bg_reset: Option<D::Color>,
    /// The color palette for looking up ANSI colors by index, including the 16 named ones.
    pub palette: Palette<'c, D::Color>,
    /// The sides of cells that should be aligned in case the set of glyph images that represent a
    /// given character or character cluster span a different number of cells than expected in the
    /// Unicode standard and Ratatui. For example, `▲` and `▼` have graphics that require cropping,
    /// and the _x_-axis anchor point determines which sections to draw.
    pub anchor_x: AnchorX,
    /// The values to carry across calls to different methods for a given frame, and across frames.
    state: State,
}

/// Backend with a reference to a draw target.
///
/// A backend or backend wrapper that implements this trait is able to call functions that expect a
/// reference with exclusive access to a draw target as an argument, and it offers extended drawing
/// capabilities that are required by wrappers in order to perform their tasks.
pub trait DrawTargetBackend<D: DrawTarget>: Backend {
    /// Invokes the specified function item, which gets to access the draw target in the scope of a
    /// function call.
    fn call(&mut self, f: impl FnMut(&mut D) -> Result<(), D::Error>) -> Result<(), D::Error>;

    /// Draws the specified content as if [`HIDDEN`](ratatui_core::style::Modifier::HIDDEN) was set
    /// on all of the items, which is equivalent to using the background colors — or the foreground
    /// colors if [`REVERSED`](ratatui_core::style::Modifier::REVERSED) is set — to clear the cells
    /// that each of the characters or character clusters spans.
    fn draw_hidden<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>;

    /// Draws the specified content using another set of colors and cropped to the specified extent
    /// with the [`Symbol::UnderCursor`] parameter; otherwise, the characters or character clusters
    /// from the [`Symbol::Custom`] parameter are drawn instead.
    ///
    /// If the content is reversed, then the set of colors for the cursor are also reversed, and if
    /// the content has text decorations, then those are also applied if [`Symbol::UnderCursor`] is
    /// drawn, as is whether text should be bold or hidden, including having _blinked_.
    ///
    /// [`Symbol::Custom`] only respects the content's modifier to have the set of colors reversed.
    fn draw_cursor<'z, I>(
        &mut self,
        content: I,
        colors: Colors,
        extent: Extent,
        symbol: Symbol,
    ) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>;

    /// Advances the blinking animation associated with the backend or backend wrapper if there are
    /// such features, calling [`advance_blink_by`](DrawTargetBackend::advance_blink_by) afterwards
    /// so that inner layers can do the same. The `ticks` are added to their internal frame counts.
    fn advance_blink_by(&mut self, ticks: usize) -> Result<(), Self::Error>;

    /// Takes the unit from the backend or backend wrapper, indicating, when [`Some`], that an area
    /// of the draw target with the cursor has been redrawn without the cursor being drawn over it.
    fn take_dirty_cursor(&mut self) -> Result<Option<()>, Self::Error>;
}

/// Backend configuration retrieval and modification.
///
/// A backend or backend wrapper that implements this trait allows its fields that are configurable
/// to have their values read or have new values assigned.
pub trait ConfigureBackend<'a, 'b, 'c, T, C>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// Returns the bitmap font to use in general.
    fn font(&self) -> &'b BitmapFont<'a, C, 1>;

    /// Sets the bitmap font to use in general.
    ///
    /// This bitmap font is used to render text that either has no modifiers set, is set to italic,
    /// and also text that is set to bold, when the bitmap font for when text should be bold is not
    /// set to a value.
    fn set_font(&mut self, font: &'b BitmapFont<'a, C, 1>);

    /// Returns the optional bitmap font for when text should be bold.
    ///
    /// When not set to a value, the backend renders all texts, including text that should be bold,
    /// using the regular font.
    fn font_bold(&self) -> Option<&'b BitmapFont<'a, C, 1>>;

    /// Sets the optional bitmap font for when text should be bold.
    ///
    /// When not set to a value, the backend renders all texts, including text that should be bold,
    /// using the regular font.
    ///
    /// This bitmap font and the regular one should have the same cell size; otherwise, the backend
    /// will introduce clipping or padding with the background color in character cells, the reason
    /// being that the cell size is always calculated using the regular font.
    fn set_font_bold(&mut self, font_bold: Option<&'b BitmapFont<'a, C, 1>>);

    /// Returns the optional foreground color to use in case no specific color is set.
    ///
    /// When not set to a value, the backend uses the inverse of the default value for type `T`.
    fn fg_reset(&self) -> Option<T>;

    /// Sets the optional foreground color to use in case no specific color is set.
    ///
    /// When not set to a value, the backend uses the inverse of the default value for type `T`.
    ///
    /// This color is used as the default foreground color, when the text color is reset.
    fn set_fg_reset(&mut self, fg_reset: Option<T>);

    /// Returns the optional background color to use in case no specific color is set.
    ///
    /// When not set to a value, the backend uses the default value for type `T`.
    fn bg_reset(&self) -> Option<T>;

    /// Sets the optional background color to use in case no specific color is set.
    ///
    /// When not set to a value, the backend uses the default value for type `T`.
    ///
    /// This color is used as the default background color, when the text background color is reset.
    fn set_bg_reset(&mut self, bg_reset: Option<T>);

    /// Returns the color palette for looking up ANSI colors by index, including the 16 named ones.
    fn palette(&self) -> Palette<'c, T>;

    /// Sets the color palette for looking up ANSI colors by index, including the 16 named ones.
    fn set_palette(&mut self, palette: Palette<'c, T>);

    /// Returns the _x_-axis anchor point for which sides of cells to align in case of disagreement.
    fn anchor_x(&self) -> AnchorX;

    /// Sets the _x_-axis anchor point for which sides of cells to align in case of disagreement.
    fn set_anchor_x(&mut self, anchor_x: AnchorX);
}

impl<'a, 'b, 'c, 'd, D, C> DumoBackend<'a, 'b, 'c, 'd, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    /// Creates a new backend with exclusive access to the specified draw target, configuring it to
    /// use the specified bitmap font for text rendering.
    pub const fn new(target: &'d mut D, font: &'b BitmapFont<'a, C, 1>) -> Self
    where
        D::Color: Palettes<'c>,
    {
        Self {
            target,
            font,
            font_bold: None,
            fg_reset: None,
            bg_reset: None,
            palette: D::Color::XTERM_256,
            anchor_x: AnchorX::Left,
            state: State::new(),
        }
    }
}

impl<'a, 'b, 'c, D, C> Backend for DumoBackend<'a, 'b, 'c, '_, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    type Error = Error<D::Error>;

    fn draw<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        use MeasureError::*;
        use embedded_graphics::geometry::Size;
        use unicode_width::UnicodeWidthStr;

        const ORIGIN: Point = Point::zero();
        const BASELINE: Baseline = Baseline::Top;

        let cell_size = self.font.cell_size();
        for (x, y, cell) in content {
            let text_color = cell.fg.map_with(self.palette).or(self.fg_reset);
            let text_color = text_color.unwrap_or(D::Color::default().invert());
            let background_color = cell.bg.map_with(self.palette).or(self.bg_reset);
            let background_color = background_color.unwrap_or_default();

            let is_dim = cell.modifier.intersects(Modifier::DIM);
            let text_color = if is_dim {
                text_color.weighted_avg(
                    background_color,
                    background_color,
                    text_color,
                    text_color,
                    background_color,
                )
            } else {
                text_color
            };

            let is_reversed = cell.modifier.intersects(Modifier::REVERSED);
            let [text_color, background_color] = if is_reversed {
                [background_color, text_color]
            } else {
                [text_color, background_color]
            };

            let is_underlined = cell.modifier.intersects(Modifier::UNDERLINED);
            let underline_color = if is_underlined {
                cell.underline_color
                    .map_with(self.palette)
                    .map(DecorationColor::Custom)
                    .unwrap_or(DecorationColor::TextColor)
            } else {
                DecorationColor::None
            };

            let is_crossed_out = cell.modifier.intersects(Modifier::CROSSED_OUT);
            let strikethrough_color = if is_crossed_out {
                DecorationColor::TextColor
            } else {
                DecorationColor::None
            };

            let mut renderer = BitmapFontStyle::new(self.font, text_color);
            renderer.background_color = Some(background_color);
            renderer.underline_color = underline_color;
            renderer.strikethrough_color = strikethrough_color;

            let text = cell.symbol();

            let columns_rows = [x, y].map(Into::into).into();
            let pixels = cell_size.checked_component_mul(columns_rows);
            let [x_offset, y_offset] = pixels.ok_or(InvalidSize)?.into();
            let top_left = Point {
                x: ORIGIN.x.saturating_add_unsigned(x_offset),
                y: ORIGIN.y.saturating_add_unsigned(y_offset),
            };

            let columns = text.width().try_into().unwrap_or(u32::MAX);
            let pixels = cell_size.width.checked_mul(columns);
            let explicit_width = pixels.ok_or(InvalidSize)?;
            let size = Size::new(explicit_width, renderer.line_height());
            let clip_area = Rectangle { top_left, size };
            let mut adapter = self.target.clipped(&clip_area);

            let is_hidden = cell.modifier.intersects(Modifier::HIDDEN);
            if is_hidden {
                self.target
                    .fill_solid(&clip_area, background_color)
                    .map_err(Error::Draw)?;
            } else if text.chars().all(|char| char::is_ascii_whitespace(&char)) {
                renderer
                    .draw_whitespace(explicit_width, top_left, BASELINE, &mut adapter)
                    .map_err(Error::Draw)?;
            } else {
                let metrics = renderer.measure_string(text, top_left, BASELINE);
                let pixels = metrics.next_position.x.saturating_sub(top_left.x);
                let inherent_width = pixels.try_into().unwrap_or_default();
                let crop_area = clip_area.resized_width(inherent_width, self.anchor_x);
                let mut adapter = adapter.translated(crop_area.top_left);

                let is_bold = cell.modifier.intersects(Modifier::BOLD);
                if is_bold && let Some(font_bold) = self.font_bold {
                    renderer.font = font_bold;

                    let width = explicit_width.saturating_sub(inherent_width);
                    let next_position = renderer
                        .draw_string(text, Point::zero(), BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    renderer
                        .draw_whitespace(width, next_position, BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    let metrics = renderer.measure_string(text, crop_area.top_left, BASELINE);
                    let below = clip_area.below(&metrics.bounding_box);
                    let wide = clip_area.above(&below);
                    let left = wide.left_of(&metrics.bounding_box);
                    let right = metrics.next_position.x.saturating_add_unsigned(width);
                    let right = wide.indent_to(right);
                    for area in [left, right, below] {
                        self.target
                            .fill_solid(&area, background_color)
                            .map_err(Error::Draw)?;
                    }
                } else {
                    let width = explicit_width.saturating_sub(inherent_width);
                    let next_position = renderer
                        .draw_string(text, Point::zero(), BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    renderer
                        .draw_whitespace(width, next_position, BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    self.target
                        .fill_solid(&clip_area.left_of(&crop_area), background_color)
                        .map_err(Error::Draw)?;
                }
            }
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        let columns_rows = self.state.cursor_position;

        Ok(columns_rows)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        use SetCursorError::*;

        let position = position.into();
        let columns_rows = self.size()?;
        let [columns, rows] = [columns_rows.width, columns_rows.height];

        self.state.cursor_position = (position.x < columns && position.y < rows)
            .then_some(position)
            .ok_or(InvalidPosition)?;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        let background_color = self.bg_reset.unwrap_or_default();

        self.target.clear(background_color).map_err(Error::Clear)?;
        self.state.cursor_coverage = None;

        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        use MeasureError::*;

        const ORIGIN: Point = Point::zero();

        let background_color = self.bg_reset.unwrap_or_default();
        let cursor_coverage = if let Some(cursor_coverage) = self.state.cursor_coverage {
            cursor_coverage
        } else {
            let cell_size = self.font.cell_size();

            let Position { x, y } = self.state.cursor_position;
            let columns_rows = [x, y].map(Into::into).into();
            let pixels = cell_size.checked_component_mul(columns_rows);
            let [x_offset, y_offset] = pixels.ok_or(InvalidSize)?.into();
            let top_left = Point {
                x: ORIGIN.x.saturating_add_unsigned(x_offset),
                y: ORIGIN.y.saturating_add_unsigned(y_offset),
            };

            Rectangle {
                top_left,
                size: cell_size,
            }
        };

        let top = cursor_coverage.top_left.y;
        let bottom = top.saturating_add_unsigned(cursor_coverage.size.height);
        let all_pixels = self.target.bounding_box();
        let current_line = all_pixels.y_reduce(top, bottom);
        let region_areas = match clear_type {
            ClearType::All => &[all_pixels][..],
            ClearType::AfterCursor => {
                let below = all_pixels.below(&current_line);
                let right = current_line.right_of(&cursor_coverage);

                &[cursor_coverage, right, below]
            }
            ClearType::BeforeCursor => {
                let above = all_pixels.above(&current_line);
                let left = current_line.left_of(&cursor_coverage);

                &[cursor_coverage, left, above]
            }
            ClearType::CurrentLine => &[current_line],
            ClearType::UntilNewLine => {
                let right = current_line.right_of(&cursor_coverage);

                &[cursor_coverage, right]
            }
        };

        for area in region_areas {
            self.target
                .fill_solid(area, background_color)
                .map_err(Error::Clear)?;
        }

        self.state.cursor_coverage = None;

        Ok(())
    }

    fn size(&self) -> Result<Size, Self::Error> {
        use MeasureError::*;

        let target_size = self.target.bounding_box().size;
        let cell_size = self.font.cell_size();

        let columns_rows = target_size.checked_component_div_ceil(cell_size);
        let [columns, rows] = columns_rows.ok_or(InvalidSize)?.into();
        let [columns, rows] = [columns, rows].map(|count| count.try_into().map_err(TryFromSize));

        let columns_rows = Size::new(columns?, rows?);

        Ok(columns_rows)
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        use MeasureError::*;

        let target_size = self.target.bounding_box().size;
        let cell_size = self.font.cell_size();

        let columns_rows = target_size.checked_component_div_ceil(cell_size);
        let [columns, rows] = columns_rows.ok_or(InvalidSize)?.into();
        let [columns, rows] = [columns, rows].map(|count| count.try_into().map_err(TryFromSize));

        let width_height = target_size.checked_component_next_multiple_of(cell_size);
        let [width, height] = width_height.ok_or(InvalidSize)?.into();
        let [width, height] = [width, height].map(|value| value.try_into().map_err(TryFromSize));

        let window_size = WindowSize {
            columns_rows: Size::new(columns?, rows?),
            pixels: Size::new(width?, height?),
        };

        Ok(window_size)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, 'b, 'c, D, C> DrawTargetBackend<D> for DumoBackend<'a, 'b, 'c, '_, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    fn call(&mut self, mut f: impl FnMut(&mut D) -> Result<(), D::Error>) -> Result<(), D::Error> {
        f(self.target)
    }

    fn draw_hidden<'z, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        use MeasureError::*;
        use embedded_graphics::geometry::Size;
        use unicode_width::UnicodeWidthStr;

        const ORIGIN: Point = Point::zero();

        let cell_size = self.font.cell_size();
        for (x, y, cell) in content {
            let text_color = cell.fg.map_with(self.palette).or(self.fg_reset);
            let text_color = text_color.unwrap_or(D::Color::default().invert());
            let background_color = cell.bg.map_with(self.palette).or(self.bg_reset);
            let background_color = background_color.unwrap_or_default();

            let is_dim = cell.modifier.intersects(Modifier::DIM);
            let text_color = if is_dim {
                text_color.weighted_avg(
                    background_color,
                    background_color,
                    text_color,
                    text_color,
                    background_color,
                )
            } else {
                text_color
            };

            let is_reversed = cell.modifier.intersects(Modifier::REVERSED);
            let background_color = if is_reversed {
                text_color
            } else {
                background_color
            };

            let text = cell.symbol();

            let columns_rows = [x, y].map(Into::into).into();
            let pixels = cell_size.checked_component_mul(columns_rows);
            let [x_offset, y_offset] = pixels.ok_or(InvalidSize)?.into();
            let top_left = Point {
                x: ORIGIN.x.saturating_add_unsigned(x_offset),
                y: ORIGIN.y.saturating_add_unsigned(y_offset),
            };

            let columns = text.width().try_into().unwrap_or(u32::MAX);
            let pixels = cell_size.width.checked_mul(columns);
            let explicit_width = pixels.ok_or(InvalidSize)?;
            let size = Size::new(explicit_width, self.font.metrics.line_height());
            let fill_area = Rectangle { top_left, size };

            self.target
                .fill_solid(&fill_area, background_color)
                .map_err(Error::Draw)?;
        }

        Ok(())
    }

    fn draw_cursor<'z, I>(
        &mut self,
        content: I,
        colors: Colors,
        extent: Extent,
        symbol: Symbol,
    ) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'z Cell)>,
    {
        use MeasureError::*;
        use embedded_graphics::geometry::Size;
        use unicode_width::UnicodeWidthStr;

        const ORIGIN: Point = Point::zero();
        const BASELINE: Baseline = Baseline::Top;

        let cell_size = self.font.cell_size();
        for (x, y, cell) in content {
            let [text_color, background_color] = match colors {
                Colors::ReversedReset => {
                    let text_color = self.fg_reset.unwrap_or(D::Color::default().invert());
                    let background_color = self.bg_reset.unwrap_or_default();

                    [background_color, text_color]
                }
                Colors::InvertedReset => {
                    let text_color = self.fg_reset.unwrap_or(D::Color::default().invert());
                    let text_color = text_color.invert();
                    let background_color = self.bg_reset.unwrap_or_default();
                    let background_color = background_color.invert();

                    [text_color, background_color]
                }
                Colors::Custom { fg, bg } => {
                    let text_color = fg.map_with(self.palette).or(self.fg_reset);
                    let text_color = text_color.unwrap_or(D::Color::default().invert());
                    let background_color = bg.map_with(self.palette).or(self.bg_reset);
                    let background_color = background_color.unwrap_or_default();

                    [text_color, background_color]
                }
            };

            let is_reversed = cell.modifier.intersects(Modifier::REVERSED);
            let [text_color, background_color] = if is_reversed {
                [background_color, text_color]
            } else {
                [text_color, background_color]
            };

            let is_underlined = cell.modifier.intersects(Modifier::UNDERLINED);
            let underline_color = if is_underlined && symbol == Symbol::UnderCursor {
                cell.underline_color
                    .map_with(self.palette)
                    .map(DecorationColor::Custom)
                    .unwrap_or(DecorationColor::TextColor)
            } else {
                DecorationColor::None
            };

            let is_crossed_out = cell.modifier.intersects(Modifier::CROSSED_OUT);
            let strikethrough_color = if is_crossed_out && symbol == Symbol::UnderCursor {
                DecorationColor::TextColor
            } else {
                DecorationColor::None
            };

            let mut renderer = BitmapFontStyle::new(self.font, text_color);
            renderer.background_color = Some(background_color);
            renderer.underline_color = underline_color;
            renderer.strikethrough_color = strikethrough_color;

            let text = cell.symbol();

            let columns_rows = [x, y].map(Into::into).into();
            let pixels = cell_size.checked_component_mul(columns_rows);
            let [x_offset, y_offset] = pixels.ok_or(InvalidSize)?.into();
            let top_left = Point {
                x: ORIGIN.x.saturating_add_unsigned(x_offset),
                y: ORIGIN.y.saturating_add_unsigned(y_offset),
            };

            let columns = text.width().try_into().unwrap_or(u32::MAX);
            let pixels = cell_size.width.checked_mul(columns);
            let explicit_width = pixels.ok_or(InvalidSize)?;
            let size = Size::new(explicit_width, renderer.line_height());
            let text_area = Rectangle { top_left, size };
            let clip_area = match extent {
                Extent::FullBlock => text_area,
                Extent::VerticalBar { width } => {
                    let explicit_width = explicit_width.min(width);

                    text_area.resized_width(explicit_width, AnchorX::Left)
                }
                Extent::Underline { height } => {
                    let top = renderer.font.metrics.y_offset(Baseline::Top);
                    let top = top.saturating_sub(renderer.font.underline.y_offset());
                    let top = top.saturating_add(top_left.y);
                    let bottom = top.saturating_add_unsigned(height);

                    text_area.y_reduce(top, bottom)
                }
                Extent::Custom(area) => {
                    let area = area.translate(top_left);

                    text_area.intersection(&area)
                }
            };

            let mut adapter = self.target.clipped(&clip_area);

            let text = match symbol {
                Symbol::UnderCursor => text,
                Symbol::Custom(text) => text,
            };

            let is_hidden = cell.modifier.intersects(Modifier::HIDDEN);
            if is_hidden && symbol == Symbol::UnderCursor {
                self.target
                    .fill_solid(&clip_area, background_color)
                    .map_err(Error::Draw)?;
            } else if text.chars().all(|char| char::is_ascii_whitespace(&char)) {
                renderer
                    .draw_whitespace(explicit_width, top_left, BASELINE, &mut adapter)
                    .map_err(Error::Draw)?;
            } else {
                let metrics = renderer.measure_string(text, top_left, BASELINE);
                let pixels = metrics.next_position.x.saturating_sub(top_left.x);
                let inherent_width = pixels.try_into().unwrap_or_default();
                let crop_area = text_area.resized_width(inherent_width, self.anchor_x);
                let mut adapter = adapter.translated(crop_area.top_left);

                let is_bold = cell.modifier.intersects(Modifier::BOLD);
                if is_bold
                    && let Some(font_bold) = self.font_bold
                    && symbol == Symbol::UnderCursor
                {
                    renderer.font = font_bold;

                    let width = explicit_width.saturating_sub(inherent_width);
                    let next_position = renderer
                        .draw_string(text, Point::zero(), BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    renderer
                        .draw_whitespace(width, next_position, BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    let metrics = renderer.measure_string(text, crop_area.top_left, BASELINE);
                    let below = clip_area.below(&metrics.bounding_box);
                    let wide = clip_area.above(&below);
                    let left = wide.left_of(&metrics.bounding_box);
                    let right = metrics.next_position.x.saturating_add_unsigned(width);
                    let right = wide.indent_to(right);
                    for area in [left, right, below] {
                        self.target
                            .fill_solid(&area, background_color)
                            .map_err(Error::Draw)?;
                    }
                } else {
                    let width = explicit_width.saturating_sub(inherent_width);
                    let next_position = renderer
                        .draw_string(text, Point::zero(), BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    renderer
                        .draw_whitespace(width, next_position, BASELINE, &mut adapter)
                        .map_err(Error::Draw)?;

                    self.target
                        .fill_solid(&clip_area.left_of(&crop_area), background_color)
                        .map_err(Error::Draw)?;
                }
            }

            self.state.cursor_coverage = Some(text_area);
        }

        Ok(())
    }

    fn advance_blink_by(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn take_dirty_cursor(&mut self) -> Result<Option<()>, Self::Error> {
        Ok(None)
    }
}

impl<'a, 'b, 'c, D, C> ConfigureBackend<'a, 'b, 'c, D::Color, C>
    for DumoBackend<'a, 'b, 'c, '_, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    fn font(&self) -> &'b BitmapFont<'a, C, 1> {
        self.font
    }

    fn set_font(&mut self, font: &'b BitmapFont<'a, C, 1>) {
        self.font = font;
    }

    fn font_bold(&self) -> Option<&'b BitmapFont<'a, C, 1>> {
        self.font_bold
    }

    fn set_font_bold(&mut self, font_bold: Option<&'b BitmapFont<'a, C, 1>>) {
        self.font_bold = font_bold;
    }

    fn fg_reset(&self) -> Option<D::Color> {
        self.fg_reset
    }

    fn set_fg_reset(&mut self, fg_reset: Option<D::Color>) {
        self.fg_reset = fg_reset;
    }

    fn bg_reset(&self) -> Option<D::Color> {
        self.bg_reset
    }

    fn set_bg_reset(&mut self, bg_reset: Option<D::Color>) {
        self.bg_reset = bg_reset;
    }

    fn palette(&self) -> Palette<'c, D::Color> {
        self.palette
    }

    fn set_palette(&mut self, palette: Palette<'c, D::Color>) {
        self.palette = palette;
    }

    fn anchor_x(&self) -> AnchorX {
        self.anchor_x
    }

    fn set_anchor_x(&mut self, anchor_x: AnchorX) {
        self.anchor_x = anchor_x;
    }
}
