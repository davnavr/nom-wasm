use crate::{error::ErrorSource, input::AsInput, values::VectorIter};
use core::fmt::Debug;
use nom::{error::ErrorKind, Parser};

/// Wraps a [`VectorIter`] to ensure the end of input is reached after all elements are parsed.
///
/// [WebAssembly vector]: crate::values::vector_fold()
#[must_use = "call Iterator::next() or finish()"]
pub struct FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    vector: VectorIter<'a, T, E, P>,
}

impl<'a, T, E, P> From<VectorIter<'a, T, E, P>> for FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn from(vector: VectorIter<'a, T, E, P>) -> Self {
        Self { vector }
    }
}
fn expected_eof<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> nom::Err<E> {
    nom::Err::Failure(E::from_error_kind(input, ErrorKind::Eof))
}

impl<'a, T, E, P> FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    /// Parses all of the remaining elements, consuming all input.
    ///
    /// # Errors
    ///
    /// Returns any errors from the underlying [`Parser`], or an [`ErrorKind::Eof`] if there was
    /// any remaining unconsumed input.
    pub fn finish(self) -> crate::input::Result<P, E> {
        let (remaining, parser) = self.vector.finish()?;
        if remaining.is_empty() {
            Ok(parser)
        } else {
            Err(expected_eof(remaining))
        }
    }
}

impl<'a, T, E, P> Clone for FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::from(self.vector.clone())
    }
}

impl<'a, T, E, P> AsInput<'a> for FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.vector.as_input()
    }
}

impl<'a, T, E, P> Iterator for FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = crate::input::Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.vector.next() {
            Some(_) if !self.vector.has_remaining() && !self.as_input().is_empty() => {
                Some(Err(expected_eof(self.as_input())))
            }
            result => result,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.vector.size_hint()
    }
}

impl<'a, T, E, P> core::iter::FusedIterator for FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
}

impl<'a, T, E, P> Debug for FullVectorIter<'a, T, E, P>
where
    E: ErrorSource<'a> + Debug,
    P: Parser<&'a [u8], T, E> + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.vector, f)
    }
}
