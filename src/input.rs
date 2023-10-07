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

/// Result type for parser operations that do not explicitly take and return parser input.
///
/// This contrasts with [`Parsed<'a, T>`](crate::Parsed), which returns the remaining parser input
/// on success.
pub type Result<T, E> = core::result::Result<T, nom::Err<E>>;

/// Runs the given `parser`, updating the `input` to point to the remaining bytes that have not
/// yet been parsed. If the `parser` returns an error, `input` will instead point to the empty
/// slice.
#[inline]
pub fn parse_with<'a, T, E, P>(input: &mut &'a [u8], mut parser: P) -> Result<T, E>
where
    E: crate::error::ErrorSource<'a>,
    P: nom::Parser<&'a [u8], T, E>,
{
    match parser.parse(input) {
        Ok((updated, value)) => {
            *input = updated;
            Ok(value)
        }
        Err(err) => {
            *input = &[];
            Err(err)
        }
    }
}
