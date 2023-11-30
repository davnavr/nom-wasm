use crate::error::ErrorSource;
use core::fmt::Debug;
use nom::Parser;

/// Parses a [WebAssembly vector]'s elements one at a time.
///
/// [WebAssembly vector]: crate::values::vector_fold()
pub struct VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    remaining: usize,
    input: &'a [u8],
    parser: P,
    _marker: core::marker::PhantomData<fn() -> Result<T, E>>,
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
        let (input, remaining) = crate::values::vector_length(input)?;
        Ok(Self::new(remaining, input, parser))
    }

    /// The expected remaining number of elements that have not yet been parsed.
    #[inline]
    pub fn expected_len(&self) -> usize {
        self.remaining
    }

    /// Returns `true` if there are elements that have yet to be parsed.
    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.remaining > 0
    }

    /// Consumes the [`VectorIter`], parses all remaining elements, and returns the [`Parser`] used
    /// to parse each item.
    pub fn into_parser(mut self) -> crate::Parsed<'a, P, E> {
        while crate::values::sequence::Sequence::parse(&mut self)?.is_some() {}
        Ok((self.input, self.parser))
    }

    pub(in crate::values::vector) fn ignore_remaining(&mut self) {
        self.remaining = 0;
    }

    // #[inline(never)]
    // #[cold]
    fn parse_error(&mut self, err: nom::Err<E>) -> nom::Err<E> {
        self.remaining = 0;

        let expected = core::mem::replace(&mut self.remaining, 0)
            .try_into()
            .unwrap_or(u32::MAX);

        err.map(|other| {
            E::append(self.input, crate::error::ErrorKind::Count, other).with_cause(
                crate::error::ErrorCause::Vector(crate::values::InvalidVector::Remaining {
                    expected,
                }),
            )
        })
    }
}

impl<'a, T, E, P> crate::values::Sequence<'a> for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E>,
{
    type Item = T;
    type Error = E;

    fn parse(&mut self) -> crate::input::Result<Option<T>, E> {
        // If an error occured, the remaining count is set to 0
        if let Some(next_remaining) = self.remaining.checked_sub(1) {
            match self.parser.parse(self.input) {
                Ok((input, ok)) => {
                    self.remaining = next_remaining;
                    self.input = input;
                    Ok(Some(ok))
                }
                Err(err) => Err(self.parse_error(err)),
            }
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Lower bound of 0 since an error can occur when the first item is parsed
        (0, Some(self.remaining))
    }
}

impl<'a, T, E, P> Clone for VectorIter<'a, T, E, P>
where
    E: ErrorSource<'a>,
    P: Parser<&'a [u8], T, E> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            remaining: self.remaining,
            input: self.input,
            parser: self.parser.clone(),
            _marker: core::marker::PhantomData,
        }
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
        Debug::fmt(&crate::values::SequenceDebug::from(self.clone()), f)
    }
}
