//! Types for specifying how to render text that should blink.

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

impl Blink {
    /// Creates a new blink animation cycle with the specified period, where text is first visible,
    /// then hidden for a duration that is half the period, possibly one frame less than the first,
    /// where `period` is a number that measures frame count.
    pub const fn with_period(period: usize) -> Self {
        let blink = period / 2;
        let delay = period - blink;

        Self::Repeat(delay, blink)
    }

    /// Returns whether text has _blinked_ in a given frame, specified with a zero-based index from
    /// the beginning of the animation cycle, wrapping if greater than or equal to the period.
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
