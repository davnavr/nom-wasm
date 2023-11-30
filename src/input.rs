//! Types, traits, and functions for processing parser input.

//pub trait Finish

/// Trait for obtaining parser input.
pub trait AsInput<'a> {
    /// Gets the underlying input to the parser.
    fn as_input(&self) -> &'a [u8];
}

impl<'a> AsInput<'a> for &'a [u8] {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self
    }
}

impl<'a, A: AsInput<'a>> AsInput<'a> for &A {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        A::as_input(self)
    }
}

impl<'a, A: AsInput<'a>> AsInput<'a> for &mut A {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        A::as_input(self)
    }
}

/// Result type for parser operations that do not explicitly take and return parser input.
///
/// This contrasts with [`Parsed<'a, T>`](crate::Parsed), which returns the remaining parser input
/// on success.
pub type Result<T, E> = core::result::Result<T, nom::Err<E>>;
