mod cell;
mod rect;
mod size;

use core::fmt::Debug;

use self::cell::CellSize;
use self::rect::RectangleExt;
use self::size::SizeExt;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::draw_target::DrawTargetExt;
use embedded_graphics::geometry::Point;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, DecorationColor};
use mplusfonts::BitmapFont;
use mplusfonts::color::{Invert, Screen, WeightedAvg};
use mplusfonts::style::BitmapFontStyle;
use ratatui_core::backend::{Backend, ClearType, WindowSize};
use ratatui_core::buffer::Cell;
use ratatui_core::layout::{Position, Size};
use ratatui_core::style::Modifier;

use crate::color::{MapWith, Palette, Palettes};
use crate::error::{Error, GetCursorError, MeasureError, SetCursorError};

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
}

/// Backend with a reference to a draw target.
///
/// A backend or backend wrapper that implements this trait is able to call functions that expect a
/// reference with exclusive access to a draw target as an argument.
pub trait DrawTargetBackend<F, D>: Backend
where
    D: DrawTarget,
    F: FnMut(&mut D) -> Result<(), D::Error>,
{
    /// Invoke the specified function item, having it called by the backend, passing a reference to
    /// the draw target, to which the backend holds an exclusive reference, as an argument.
    fn call(&mut self, f: &mut F) -> Result<(), D::Error>;
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
        }
    }

    pub const fn with_flush<F>(self, flush_fn: F) -> FlushWrapper<Self, F, D>
    where
        F: FnMut(&mut D) -> Result<(), D::Error>,
    {
        FlushWrapper::new(self, flush_fn)
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

        const ORIGIN: Point = Point::zero();
        const BASELINE: Baseline = Baseline::Top;

        let cell_size = self.font.cell_size();
        for (x, y, cell) in content {
            let text_color = cell.fg.map_with(self.palette).or(self.fg_reset);
            let text_color = text_color.unwrap_or(D::Color::default().invert());
            let background_color = cell.bg.map_with(self.palette).or(self.bg_reset);
            let background_color = background_color.unwrap_or_default();
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

            let metrics = renderer.measure_string(text, top_left, BASELINE);
            let line_height = renderer.line_height();
            let bottom_right = Point {
                x: metrics.next_position.x,
                y: metrics.next_position.y.saturating_add_unsigned(line_height),
            };

            let clip_area = Rectangle::with_corners(top_left, bottom_right);
            let mut adapter = self.target.clipped(&clip_area);

            let is_bold = cell.modifier.intersects(Modifier::BOLD);
            if is_bold && let Some(font_bold) = self.font_bold {
                renderer.font = font_bold;
                renderer
                    .draw_string(text, top_left, BASELINE, &mut adapter)
                    .map_err(Error::Draw)?;

                let metrics = renderer.measure_string(text, top_left, BASELINE);
                let right = clip_area.right_of(&metrics.bounding_box);
                let below = clip_area.left_of(&right).below(&metrics.bounding_box);
                for area in [right, below] {
                    self.target
                        .fill_solid(&area, background_color)
                        .map_err(Error::Draw)?;
                }
            } else {
                renderer
                    .draw_string(text, top_left, BASELINE, &mut adapter)
                    .map_err(Error::Draw)?;
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
        use GetCursorError::*;

        let [x, y] = Point::zero().into();
        let [x, y] = [x, y].map(|index| index.try_into().map_err(TryFromPoint));

        let columns_rows = Position::new(x?, y?);

        Ok(columns_rows)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        use SetCursorError::*;

        let position = position.into();
        let columns_rows = self.size()?;
        let [columns, rows] = [columns_rows.width, columns_rows.height];

        let _ = (position.x < columns && position.y < rows)
            .then_some(Point::new(position.x.into(), position.y.into()))
            .ok_or(InvalidPosition)?;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        let background_color = self.bg_reset.unwrap_or_default();

        self.target.clear(background_color).map_err(Error::Clear)?;

        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        let background_color = self.bg_reset.unwrap_or_default();

        let region = match clear_type {
            ClearType::All => self.target.bounding_box(),
            _ => Rectangle::zero(),
        };

        self.target
            .fill_solid(&region, background_color)
            .map_err(Error::Clear)?;

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

impl<'a, 'b, 'c, F, D, C> DrawTargetBackend<F, D> for DumoBackend<'a, 'b, 'c, '_, D, C>
where
    C: PixelColor + From<C::Raw>,
    D: DrawTarget,
    D::Color: PixelColor + Default + Invert + Screen + WeightedAvg + From<Rgb888>,
    D::Error: Debug,
    F: FnMut(&mut D) -> Result<(), D::Error>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
    BitmapFontStyle<'a, 'b, D::Color, C, 1>: TextRenderer<Color = D::Color>,
{
    fn call(&mut self, f: &mut F) -> Result<(), D::Error> {
        f(self.target)
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
}
