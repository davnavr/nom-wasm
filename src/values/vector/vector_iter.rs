use crate::{
    error::{self, ErrorSource},
    values, Parsed,
};
use core::fmt::Debug;
use nom::Parser;

/// Provides an [`Iterator`] implementation for parsing a [WebAssembly vector].
///
/// [WebAssembly vector]: crate::values::vector_fold()
#[must_use = "call Iterator::next() or finish()"]
pub struct VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    remaining: usize,
    input: &'a [u8],
    parser: P,
    _marker: core::marker::PhantomData<fn() -> crate::Parsed<'a, T, E>>,
}

impl<'a, T, E, P> VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    /// Creates an [`Iterator`] for parsing a vector containing `remaining` items with the given
    /// [`Parser`], from the given `input`.
    pub fn new<L: nom::ToUsize>(remaining: L, input: &'a [u8], parser: P) -> Self {
        Self {
            remaining: remaining.to_usize(),
            input,
            parser,
            _marker: core::marker::PhantomData,
        }
    }

    /// Creates an [`Iterator`] for parsing a vector from the given `input` with a parsed
    /// [*LEB128* encoded length], using the given [`Parser`].
    ///
    /// [*LEB128* encoded length]: crate::values::vector_length
    pub fn with_parsed_length(input: &'a [u8], parser: P) -> crate::input::Result<Self, E> {
        let (input, remaining) = values::vector_length(input)?;
        Ok(Self::new(remaining, input, parser))
    }

    /// Returns `true` if there are elements that have yet to be parsed.
    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.remaining > 0
    }

    /// Parses all of the remaining elements, returning the remaining `input` and the [`Parser`]
    /// used to parse the vector's elements.
    #[inline]
    pub fn finish(mut self) -> Parsed<'a, P, E> {
        for result in &mut self {
            let _ = result?;
        }

        Ok((self.input, self.parser))
    }
}

impl<'a, T, E, P> Iterator for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = crate::input::Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_remaining) = self.remaining.checked_sub(1) {
            Some(match self.parser.parse(self.input) {
                Ok((input, ok)) => {
                    self.remaining = next_remaining;
                    self.input = input;
                    Ok(ok)
                }
                Err(err) => {
                    let expected = core::mem::replace(&mut self.remaining, 0)
                        .try_into()
                        .unwrap_or(u32::MAX);

                    let error = err.map(|other| {
                        E::append(self.input, error::ErrorKind::Count, other).with_cause(
                            error::ErrorCause::Vector(values::InvalidVector::Remaining {
                                expected,
                            }),
                        )
                    });

                    self.remaining = 0;
                    self.input = &[];
                    Err(error)
                }
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            (!self.input.is_empty() || self.remaining > 0) as usize,
            Some(self.input.len().min(self.remaining)),
        )
    }
}

impl<'a, T, E, P> ExactSizeIterator for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<'a, T, E, P> core::iter::FusedIterator for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
}

impl<'a, T, E, P> Clone for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.remaining, self.input, self.parser.clone())
    }
}

impl<'a, T, E, P> crate::input::AsInput<'a> for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.input
    }
}

impl<'a, T, E, P> Debug for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a> + Debug,
    P: Parser<&'a [u8], T, E> + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&values::SequenceDebug::from(self.clone()), f)
    }
}
