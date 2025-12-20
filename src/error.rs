use core::fmt::Debug;
use core::num::TryFromIntError;

use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    #[error("failed to draw text to target")]
    Draw(T),
    #[error("failed to clear target")]
    Clear(T),
    #[error("failed to flush changes")]
    Flush(T),
    #[error(transparent)]
    Measure(#[from] MeasureError),
    #[error(transparent)]
    GetCursor(#[from] GetCursorError),
    #[error(transparent)]
    SetCursor(#[from] SetCursorError),
}

#[derive(Error, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MeasureError {
    #[error("invalid size")]
    InvalidSize,
    #[error("unable to convert size")]
    TryFromSize(TryFromIntError),
}

#[derive(Error, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GetCursorError {
    #[error("unable to convert point")]
    TryFromPoint(TryFromIntError),
}

#[derive(Error, Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetCursorError {
    #[error("invalid position")]
    InvalidPosition,
}
