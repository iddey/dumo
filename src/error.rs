//! Error types utilized by the backend and its related functions.
//!
//! Error handlers should be able to match any errors that are to be expected from this library,
//! with one of the [`Error`](enum@Error) variants being the entry point. Some of these variants
//! have an additional error type defined for a more exact identification.

use core::fmt::Debug;
use core::num::TryFromIntError;

use thiserror::Error;

/// Error that the backend may encounter, with variants that indicate which task the backend was
/// performing when the error occurred.
///
/// Variants that have a single field of type `T` are errors that originate from the draw target.
#[derive(Error, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    /// Error that occurred while drawing text elements to the target.
    #[error("failed to draw text to target")]
    Draw(T),
    /// Error that occurred while clearing the drawing region.
    #[error("failed to clear target")]
    Clear(T),
    /// Error that occurred while flushing the changes made.
    #[error("failed to flush changes")]
    Flush(T),
    /// Error that is related to the size of the bitmap font or the text area.
    #[error(transparent)]
    Measure(#[from] MeasureError),
    /// Error that occurred while changing the position of the cursor.
    #[error(transparent)]
    SetCursor(#[from] SetCursorError),
    /// Error that occurred while changing the frame count of the cursor.
    #[error(transparent)]
    AdvanceCursorBlinking(#[from] AdvanceCursorBlinkingError),
}

/// Error that occurs while measuring the drawing region or the text area, indicating an invalid
/// backend configuration.
#[derive(Error, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MeasureError {
    /// The bitmap font has a size that the backend cannot use to calculate the size of the text
    /// area in the drawing region.
    #[error("invalid size")]
    InvalidSize,
    /// The size has numeric components that are too large.
    #[error("unable to convert size")]
    TryFromSize(TryFromIntError),
}

/// Error that occurs when attempting to set the cursor to an invalid position in the text area.
#[derive(Error, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetCursorError {
    /// The position would result in the cursor being located outside of the text area.
    #[error("invalid position")]
    InvalidPosition,
}

/// Error that occurs when attempting to advance the frames and set whether a cursor has _blinked_.
#[derive(Error, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdvanceCursorBlinkingError {
    /// The _blinked_ state is not present in any of the frames for a given animation cycle.
    #[error("invalid blinked")]
    InvalidBlinked,
}
