use crate::{
    error::{AddCause as _, ErrorCause, ErrorSource},
    input::Result,
    sequence::{self, Sequence},
    Parsed,
};
use core::fmt::Debug;
use nom::Parser;

/// Represents a [vector] in the binary format, which is a [`Sequence`] of items that can be
/// decoded by applying a [`Parser`] multiple times.
///
/// [vector]: https://webassembly.github.io/spec/core/binary/conventions.html#vectors
#[must_use = "items are parsed lazily, call Sequence::next or VectorParser::finish"]
pub struct VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    remaining: usize,
    parser: P,
    input: &'a [u8],
    _marker: core::marker::PhantomData<fn() -> Parsed<'a, T, E>>,
}

impl<'a, T, E, P> Default for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Default,
{
    #[inline]
    fn default() -> Self {
        Self::new(&[], 0, P::default())
    }
}

impl<'a, T, E, P> VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    pub(crate) fn new(input: &'a [u8], remaining: usize, parser: P) -> Self {
        Self {
            input,
            remaining,
            parser,
            _marker: core::marker::PhantomData,
        }
    }

    //pub
    fn parse_length_32_with(input: &'a [u8], parser: P) -> Result<Self, E> {
        let (input, length) =
            crate::values::leb128_u32(input).add_cause(ErrorCause::VectorLength)?;
        Ok(Self::new(input, nom::ToUsize::to_usize(&length), parser))
    }

    /// Parses all of the remaining items in the vector, ignoring the results of the underlying
    /// [`Parser`],
    pub fn finish(mut self) -> crate::Parsed<'a, P, E> {
        // TODO: Move this to a default method in Vector trait
        while self.next().is_some() {}
        Ok((self.input, self.parser))
    }
}

impl<'a, T, E, P> VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Default,
{
    /// Creates a vector with `length` items that are parsed from the given `input`.
    #[inline]
    pub fn with_length_32(length: u32, input: &'a [u8]) -> Self {
        Self::new(input, nom::ToUsize::to_usize(&length), P::default())
    }

    /// Parses a [*LEB128*](crate::values::leb128) encoded unsigned 32-bit length for a vector.
    ///
    /// # Errors
    ///
    /// Returns an error from an [`ErrorCause::VectorLength`] if the vector length could not be
    /// parsed.
    #[inline]
    pub fn parse_length_32(input: &'a [u8]) -> Result<Self, E> {
        Self::parse_length_32_with(input, P::default())
    }
}

impl<'a, T, E, P> sequence::Sequence<'a, E> for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = T;

    fn next(&mut self) -> Option<Result<Self::Item, E>> {
        let remaining = self.remaining.checked_sub(1)?;
        Some(match self.parser.parse(self.input) {
            Ok((input, value)) => {
                self.input = input;
                self.remaining = remaining;
                Ok(value)
            }
            Err(error) => {
                self.input = &[];
                self.remaining = 0;
                Err(error)
            }
        })
    }

    // TODO: Move this function to sequence module, might be commonly used
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            !self.input.is_empty() as usize,
            Some(self.remaining.min(self.input.len())),
        )
    }
}

impl<'a, T, E, P> sequence::Vector<'a, E> for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn expected_len(&self) -> usize {
        self.remaining
    }
}

impl<'a, T, E, P> IntoIterator for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a> + 'a,
    P: Parser<&'a [u8], T, E>,
{
    type Item = Result<T, E>;
    type IntoIter = sequence::Iter<'a, Self, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        sequence::Iter::wrap(self)
    }
}

impl<'a, T, E, P> Clone for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.input, self.remaining, self.parser.clone())
    }
}

/*
impl<'i, T, E, P> debug::DebugParse<'i> for VectorParser<'i, T, crate::error::Error<'i>, P>
where
    P: Parser<&'i [u8], T, E>,
    T: Debug,
{
    fn format<'a, 'b: 'a>(self, f: &'a mut core::fmt::Formatter<'b>) -> debug::Result<'i, 'a, 'b> {
        let mut list = f.debug_list();
        for result in self.into_iter() {
            match result {
                Ok(item) => {
                    list.entry(&item);
                } // TODO: Would it just be easier to allow custom E in debug::Result?
                Err(error) => return Err(debug::ParseFailed::new(list, error).into()),
            }
        }
        list.finish().map_err(Into::into)
    }
}
*/

impl<'a, T, E, P> Debug for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a> + Debug + 'a,
    P: Parser<&'a [u8], T, E> + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(self.clone().into_iter().map(crate::debug::FmtResult))
            .finish()
    }
}

impl<'a, T, E, P> crate::input::AsInput<'a> for VectorParser<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.input
    }
}
