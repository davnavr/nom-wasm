use crate::{error::ErrorSource, values::VectorIter};
use core::fmt::Debug;
use nom::{Parser, ToUsize};

/// Wraps a [`VectorIter`] to enforce a minimum number of elements when parsing a vector.
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
        crate::error::ErrorKind::Verify,
        crate::error::ErrorCause::Vector(crate::values::InvalidVector::Remaining {
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
        if vector.expected_len() < MIN.to_usize() {
            Err(nom::Err::Failure(minimum_bounds_error::<'_, MIN, E>(
                crate::input::AsInput::as_input(&vector),
                vector.expected_len(),
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
        if vector.expected_len() < MIN.to_usize() {
            Err(nom::Err::Failure(minimum_bounds_error::<'_, MIN, E>(
                input,
                vector.expected_len(),
            )))
        } else {
            Ok(Self { vector })
        }
    }

    /// Parses all of the remaining items and returns the underlying [`Parser`].
    ///
    /// See the documentation for [`VectorIter::into_parser()`] for more information.
    #[inline]
    pub fn into_parser(self) -> crate::Parsed<'a, P, E> {
        self.vector.into_parser()
    }

    /// See [`VectorIter::expected_len()`].
    pub fn expected_len(&self) -> usize {
        self.vector.expected_len()
    }
}

impl<'a, const MIN: u32, T, E, P> crate::values::Sequence<'a>
    for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = T;
    type Error = E;

    #[inline]
    fn parse(&mut self) -> crate::input::Result<Option<T>, E> {
        self.vector.parse()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.vector.size_hint()
    }
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

impl<'a, const MIN: u32, T, E, P> crate::input::AsInput<'a> for BoundedVectorIter<'a, MIN, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        crate::input::AsInput::as_input(&self.vector)
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
