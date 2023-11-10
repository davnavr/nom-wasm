use crate::{
    error::{self, ErrorSource},
    input::AsInput,
    values::VectorIter,
    Parsed,
};
use core::fmt::Debug;
use nom::{Parser, ToUsize};

/// Wraps a [`VectorIter`] to enforce a minimum number of elements when parsing a vector.
#[must_use = "call Iterator::next() or finish()"]
#[repr(transparent)]
pub struct BoundedVectorIter<'a, const MIN: u32, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    vector: VectorIter<'a, T, E, P>,
}

fn minimum_bounds_error<'a, const MIN: u32, E>(input: &'a [u8], actual: usize) -> E
where
    E: ErrorSource<'a>,
{
    E::from_error_kind_and_cause(
        input,
        error::ErrorKind::Verify,
        error::ErrorCause::Vector(crate::values::InvalidVector::Remaining {
            expected: (MIN.to_usize() - actual).try_into().unwrap_or(u32::MAX),
        }),
    )
}

#[allow(missing_docs)]
impl<'a, const MIN: u32, T, E, P> BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    pub fn from_vector_iter(vector: VectorIter<'a, T, E, P>) -> crate::input::Result<Self, E> {
        if vector.len() < MIN.to_usize() {
            Err(nom::Err::Failure(minimum_bounds_error::<'_, MIN, E>(
                vector.as_input(),
                vector.len(),
            )))
        } else {
            Ok(Self { vector })
        }
    }

    /// Parses a WebAssembly vector, checking that it contains at least `MIN` elements.
    ///
    /// # Errors
    ///
    /// See the documentation for [`VectorIter::with_parsed_length()`] for more information.
    #[inline]
    pub fn with_parsed_length(input: &'a [u8], parser: P) -> crate::input::Result<Self, E> {
        let vector = VectorIter::with_parsed_length(input, parser)?;
        if vector.len() < MIN.to_usize() {
            Err(nom::Err::Failure(minimum_bounds_error::<'_, MIN, E>(
                input,
                vector.len(),
            )))
        } else {
            Ok(Self { vector })
        }
    }

    #[inline]
    pub fn finish(self) -> Parsed<'a, P, E> {
        self.vector.finish()
    }
}

impl<'a, const MIN: u32, T, E, P> Iterator for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = crate::input::Result<T, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.vector.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.vector.size_hint()
    }
}

impl<'a, const MIN: u32, T, E, P> ExactSizeIterator for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn len(&self) -> usize {
        self.vector.len()
    }
}

impl<'a, const MIN: u32, T, E, P> core::iter::FusedIterator for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
}

impl<'a, const MIN: u32, T, E, P> Clone for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Clone + Parser<&'a [u8], T, E>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            vector: self.vector.clone(),
        }
    }
}

impl<'a, const MIN: u32, T, E, P> AsInput<'a> for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.vector.as_input()
    }
}

impl<'a, const MIN: u32, T, E, P> Debug for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a> + Debug,
    P: Parser<&'a [u8], T, E> + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.vector, f)
    }
}
