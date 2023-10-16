use crate::{
    error::{self, AddCause as _, ErrorSource},
    Parsed,
};
use nom::Parser;

/// Parses a [*LEB128* encoded unsigned 32-bit integer] length which prefixes a [`vector`]'s elements.
///
/// [*LEB128* encoded unsigned 32-bit integer]: crate::values::leb128_u32
pub fn vector_length<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, usize, E> {
    crate::values::leb128_u32(input)
        .add_cause(error::ErrorCause::VectorLength)
        .map(|(input, len)| (input, nom::ToUsize::to_usize(&len)))
}

pub(crate) fn sequence<'a, E, P>(
    mut input: &'a [u8],
    count: usize,
    mut parser: P,
) -> Parsed<'a, (), E>
where
    E: ErrorSource<'a>,
    P: nom::Parser<&'a [u8], (), E>,
{
    for _ in 0..count {
        match parser.parse(input) {
            Ok((remaining, ())) => input = remaining,
            Err(nom::Err::Error(e)) => {
                return Err(nom::Err::Error(E::append(
                    input,
                    error::ErrorKind::Count,
                    e,
                )))
            }
            Err(err) => return Err(err),
        }
    }

    Ok((input, ()))
}

/// Parses a [WebAssembly vector], which is a sequence of elements prefixed by a [`u32` length].
///
/// [WebAssembly vector]: https://webassembly.github.io/spec/core/binary/conventions.html#vectors
/// [`u32` length]: vector_length
pub fn vector<'a, E, P>(input: &'a [u8], parser: P) -> Parsed<'a, (), E>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], (), E>,
{
    let (input, count) = vector_length(input)?;
    sequence(input, count, parser)
}

/// Provides an [`Iterator`] implementation for parsing a [WebAssembly vector](vector).
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
    pub fn new(remaining: usize, input: &'a [u8], parser: P) -> Self {
        Self {
            remaining,
            input,
            parser,
            _marker: core::marker::PhantomData,
        }
    }

    /// Creates an [`Iterator`] for parsing a vector from the given `input` with a parsed
    /// [*LEB128* encoded length], using the given [`Parser`].
    ///
    /// [*LEB128* encoded length]: vector_length
    pub fn with_parsed_length(input: &'a [u8], parser: P) -> crate::input::Result<Self, E> {
        let (input, remaining) = vector_length(input)?;
        Ok(Self::new(remaining, input, parser))
    }

    /// Gets the remaining `input` and the [`Parser`] used to parse the vector's elements.
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
                    self.remaining = 0;
                    Err(err)
                }
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            !self.input.is_empty() as usize,
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

impl<'a, T, E, P> core::fmt::Debug for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VectorIter")
            .field("remaining", &self.remaining)
            .field("input", &crate::hex::Bytes(self.input))
            .finish_non_exhaustive()
    }
}
