pub mod flush;

/// Wrapper around an arbitrary object.
///
/// A backend wrapper that implements this trait is subject to blanket implementations that forward
/// function calls to a backend or another backend wrapper, given that the backend implements those
/// functions in the first place.
pub trait Wrapper {
    /// The type of the inner object.
    type Inner;

    /// Returns a reference to the inner object.
    fn inner(&self) -> &Self::Inner;

    /// Returns a reference with exclusive access to the inner object.
    fn inner_mut(&mut self) -> &mut Self::Inner;

    /// Consumes the wrapper, returning the inner object.
    fn into_inner(self) -> Self::Inner;
}
