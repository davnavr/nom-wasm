use crate::{error::ErrorSource, values::Vector};
use core::fmt::Debug;
use nom::{error::ErrorKind, Parser};

/// Wraps a [`Vector`] to ensure the end of input is reached after all elements are parsed.
///
/// [WebAssembly vector]: crate::values::vector_fold()
pub struct FullVector<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    vector: Vector<'a, T, E, P>,
}

impl<'a, T, E, P> From<Vector<'a, T, E, P>> for FullVector<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn from(vector: Vector<'a, T, E, P>) -> Self {
        Self { vector }
    }
}

fn expected_eof<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> nom::Err<E> {
    nom::Err::Failure(E::from_error_kind(input, ErrorKind::Eof))
}

impl<'a, T, E, P> FullVector<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    /// Parses all of the remaining elements, consuming all input and returning the underlying [`Parser`].
    ///
    /// # Errors
    ///
    /// Returns any errors from the underlying [`Parser`], or an [`ErrorKind::Eof`] if there was
    /// any remaining unconsumed input.
    pub fn into_parser(self) -> crate::input::Result<P, E> {
        let (remaining, parser) = self.vector.into_parser()?;
        if remaining.is_empty() {
            Ok(parser)
        } else {
            Err(expected_eof(remaining))
        }
    }
}

impl<'a, T, E, P> Clone for FullVector<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::from(self.vector.clone())
    }
}

impl<'a, T, E, P> crate::input::AsInput<'a> for FullVector<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        crate::input::AsInput::as_input(&self.vector)
    }
}

impl<'a, T, E, P> crate::values::Sequence<'a> for FullVector<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = T;
    type Error = E;

    fn parse(&mut self) -> crate::input::Result<Option<Self::Item>, Self::Error> {
        let remaining_input = crate::input::AsInput::as_input(&self);
        if !self.vector.has_remaining() && !remaining_input.is_empty() {
            self.vector.ignore_remaining();
            Err(expected_eof(remaining_input))
        } else {
            crate::values::Sequence::parse(&mut self.vector)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.vector.size_hint()
    }
}

impl<'a, T, E, P> Debug for FullVector<'a, T, E, P>
where
    E: ErrorSource<'a> + Debug,
    P: Parser<&'a [u8], T, E> + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.vector, f)
    }
}
