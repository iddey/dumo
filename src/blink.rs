//! Types for specifying how to render text that should blink.

use core::fmt::Debug;

use embedded_graphics::draw_target::DrawTarget;
use ratatui_core::backend::Backend;

use crate::backend::DrawTargetBackend;
use crate::wrapper::WrapTrait;
use crate::wrapper::traits;

/// Blinked state representation. If the inner value is `true`, then text should be hidden, `false`
/// means that text is visible and drawn to the target because it has not _blinked_.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Blinked(pub bool);

/// Blink animation definition. At the moment, blinking is always a 50%–50% split between frames of
/// text being visible and hidden, where the odd frame is added to the delay, when text is visible.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Blink {
    /// A period split into two parts, the first part is a delay before having _blinked_, while the
    /// second one is the remainder of the period, when text has _blinked_.
    Repeat(usize, usize),
}

/// Blinking animation controls. Useful in case a [`Terminal`] invokes the [`Backend::draw`] method
/// a varying number of times per frame, or just more than once per what should count as one frame.
///
/// [`Terminal`]: ratatui_core::terminal::Terminal
pub trait ControlBlinking<D: DrawTarget>
where
    Self: DrawTargetBackend<D>,
{
    /// Returns `true` if calls to the [`Backend::draw`] method also advance the blinking animation
    /// every time.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(all(feature = "alloc", feature = "font-8x24", feature = "font-4-bits")))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::{Blink, ControlBlinking};
    /// use dumo::cursor::{Colors, Cursor, Extent};
    /// use dumo::fonts::*;
    /// use dumo::DumoBackend;
    /// # use embedded_graphics::mock_display::MockDisplay;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// # let mut display: MockDisplay<Rgb565> = MockDisplay::new();
    /// let backend = DumoBackend::new(&mut display, &FONT_8X24_4_BITS)
    ///     .with_blink(Blink::with_period(16), Blink::with_period(8))
    ///     .with_cursor(
    ///         Cursor::default()
    ///             .blink(Blink::with_period(10))
    ///             .colors(Colors::InvertedReset)
    ///             .extent(Extent::Underline { height: 2 }),
    ///     );
    ///
    /// let is_blinking = backend.blinking();
    /// ```
    fn blinking(&self) -> bool;

    /// Starts driving the blinking animation, stepping by one frame at the end of each call to the
    /// [`Backend::draw`] method. This is the default behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(all(feature = "alloc", feature = "font-8x24", feature = "font-4-bits")))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::{Blink, ControlBlinking};
    /// use dumo::cursor::{Colors, Cursor, Extent};
    /// use dumo::fonts::*;
    /// use dumo::DumoBackend;
    /// # use embedded_graphics::mock_display::MockDisplay;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// # let mut display: MockDisplay<Rgb565> = MockDisplay::new();
    /// let mut backend = DumoBackend::new(&mut display, &FONT_8X24_4_BITS)
    ///     .with_blink(Blink::with_period(16), Blink::with_period(8))
    ///     .with_cursor(
    ///         Cursor::default()
    ///             .blink(Blink::with_period(10))
    ///             .colors(Colors::InvertedReset)
    ///             .extent(Extent::Underline { height: 2 }),
    ///     );
    ///
    /// backend.stop_blinking();
    /// backend.start_blinking();
    /// ```
    fn start_blinking(&mut self);

    /// Stops the blinking animation being driven by calls to the [`Backend::draw`] method, meaning
    /// that [`advance_blink`] needs to be called every frame.
    ///
    /// [`advance_blink`]: ControlBlinking::advance_blink
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(all(feature = "alloc", feature = "font-8x24", feature = "font-4-bits")))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::{Blink, ControlBlinking};
    /// use dumo::cursor::{Colors, Cursor, Extent};
    /// use dumo::fonts::*;
    /// use dumo::DumoBackend;
    /// # use embedded_graphics::mock_display::MockDisplay;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// # let mut display: MockDisplay<Rgb565> = MockDisplay::new();
    /// let mut backend = DumoBackend::new(&mut display, &FONT_8X24_4_BITS)
    ///     .with_blink(Blink::with_period(16), Blink::with_period(8))
    ///     .with_cursor(
    ///         Cursor::default()
    ///             .blink(Blink::with_period(10))
    ///             .colors(Colors::InvertedReset)
    ///             .extent(Extent::Underline { height: 2 }),
    ///     );
    ///
    /// backend.stop_blinking();
    /// ```
    fn stop_blinking(&mut self);

    /// Advances the blink animation cycles for all of the backend wrappers, stepping by one frame.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(all(feature = "alloc", feature = "font-8x24", feature = "font-4-bits")))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::{Blink, ControlBlinking};
    /// use dumo::cursor::{Colors, Cursor, Extent};
    /// use dumo::fonts::*;
    /// use dumo::DumoBackend;
    /// # use embedded_graphics::mock_display::MockDisplay;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// # let mut display: MockDisplay<Rgb565> = MockDisplay::new();
    /// let mut backend = DumoBackend::new(&mut display, &FONT_8X24_4_BITS)
    ///     .with_blink(Blink::with_period(16), Blink::with_period(8))
    ///     .with_cursor(
    ///         Cursor::default()
    ///             .blink(Blink::with_period(10))
    ///             .colors(Colors::InvertedReset)
    ///             .extent(Extent::Underline { height: 2 }),
    ///     );
    ///
    /// backend.stop_blinking();
    ///
    /// for _ in 0..100 {
    ///     backend.advance_blink();
    /// }
    /// ```
    fn advance_blink(&mut self) -> Result<(), Self::Error> {
        self.advance_blink_by(1)
    }
}

/// Blinking animation controls for a cursor. Useful for revealing the cursor at its new position
/// whenever it moves, after a [`Terminal`] invokes the [`Backend::set_cursor_position`] method.
///
/// [`Terminal`]: ratatui_core::terminal::Terminal
pub trait ControlCursorBlinking
where
    Self: Backend,
{
    /// Advances the blink animation cycle for the cursor if its _blinked_ state does not match the
    /// specified state, stepping forward until it does, or returning an error if there is no match
    /// in any of the frames.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(all(feature = "alloc", feature = "font-8x24", feature = "font-4-bits")))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::{Blink, Blinked, ControlCursorBlinking};
    /// use dumo::cursor::{Colors, Cursor, Extent};
    /// use dumo::fonts::*;
    /// use dumo::DumoBackend;
    /// # use embedded_graphics::mock_display::MockDisplay;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    ///
    /// # let mut display: MockDisplay<Rgb565> = MockDisplay::new();
    /// let mut backend = DumoBackend::new(&mut display, &FONT_8X24_4_BITS)
    ///     .with_blink(Blink::with_period(16), Blink::with_period(8))
    ///     .with_cursor(
    ///         Cursor::default()
    ///             .blink(Blink::with_period(10))
    ///             .colors(Colors::InvertedReset)
    ///             .extent(Extent::Underline { height: 2 }),
    ///     );
    ///
    /// for _ in 0..100 {
    ///     backend.advance_cursor_blink_to(Blinked(true));
    /// }
    /// ```
    fn advance_cursor_blink_to(&mut self, blinked: Blinked) -> Result<(), Self::Error>;
}

impl Blink {
    /// Creates a new blink animation cycle with the specified period, where text is first visible,
    /// then hidden for a duration that is half the period, possibly one frame less than the first,
    /// where `period` is a number that measures frame count.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(feature = "alloc"))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::Blink;
    ///
    /// let blink = Blink::with_period(10); // Repeat(5, 5)
    /// ```
    pub const fn with_period(period: usize) -> Self {
        let blink = period / 2;
        let delay = period - blink;

        Self::Repeat(delay, blink)
    }

    /// Returns whether text has _blinked_ in a given frame, specified with a zero-based index from
    /// the beginning of the animation cycle, wrapping if greater than or equal to the period.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(not(feature = "alloc"))]
    /// # {
    /// #     compile_error!("doc-test is missing required features");
    /// # }
    /// #
    /// use dumo::blink::{Blink, Blinked};
    ///
    /// let blink = Blink::with_period(10); // Repeat(5, 5)
    ///
    /// assert_eq!(blink.get(0), Blinked(false)); // Text is drawn as normal
    /// assert_eq!(blink.get(5), Blinked(true)); // Text is not drawn or is drawn over
    /// ```
    pub const fn get(&self, index: usize) -> Blinked {
        match *self {
            Self::Repeat(delay, blink) => {
                if let Some(period) = delay.checked_add(blink) {
                    if let Some(index) = index.checked_rem(period) {
                        Blinked(index >= delay)
                    } else {
                        Blinked(true)
                    }
                } else {
                    Blinked(index >= delay)
                }
            }
        }
    }
}

/// Blanket implementation of the [`ControlBlinking`] trait for function call passthrough.
impl<W, B, D> ControlBlinking<D> for W
where
    B: ControlBlinking<D>,
    D: DrawTarget,
    D::Error: Debug,
    W: DrawTargetBackend<D, Error = B::Error>,
    W: WrapTrait<traits::ControlBlinking, Inner = B>,
{
    fn blinking(&self) -> bool {
        self.inner().blinking()
    }

    fn start_blinking(&mut self) {
        self.inner_mut().start_blinking();
    }

    fn stop_blinking(&mut self) {
        self.inner_mut().stop_blinking();
    }

    fn advance_blink(&mut self) -> Result<(), Self::Error> {
        self.inner_mut().advance_blink()
    }
}

/// Blanket implementation of the [`ControlCursorBlinking`] trait for function call passthrough.
impl<W, B> ControlCursorBlinking for W
where
    B: ControlCursorBlinking,
    W: Backend<Error = B::Error>,
    W: WrapTrait<traits::ControlCursorBlinking, Inner = B>,
{
    fn advance_cursor_blink_to(&mut self, blinked: Blinked) -> Result<(), Self::Error> {
        self.inner_mut().advance_cursor_blink_to(blinked)
    }
}
