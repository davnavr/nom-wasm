use crate::{
    error::{self, AddCause as _, ErrorSource},
    Parsed,
};
use nom::Parser;

mod bounded_vector_iter;
mod vector_iter;

pub use bounded_vector_iter::BoundedVectorIter;
pub use vector_iter::VectorIter;

/// Describes why a WebAssembly vector could not be parsed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum InvalidVector {
    #[non_exhaustive]
    Length,
    #[non_exhaustive]
    Remaining { expected: u32 },
}

crate::static_assert::check_size!(InvalidVector, <= 8);

impl core::fmt::Display for InvalidVector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Length => f.write_str("expected item count prefix for vector"),
            Self::Remaining { expected } => write!(f, "expected {expected} more items in vector"),
        }
    }
}

/// Parses a [*LEB128* encoded unsigned 32-bit integer] length which prefixes a [`vector`]'s elements.
///
/// [*LEB128* encoded unsigned 32-bit integer]: crate::values::leb128_u32
pub fn vector_length<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, u32, E> {
    crate::values::leb128_u32(input)
        .add_cause_with(|| error::ErrorCause::Vector(InvalidVector::Length))
}

fn sequence_inner<'a, E, P>(mut input: &'a [u8], count: usize, mut parser: P) -> Parsed<'a, (), E>
where
    E: ErrorSource<'a>,
    P: nom::Parser<&'a [u8], (), E>,
{
    for i in 0..count {
        match parser.parse(input) {
            Ok((remaining, ())) => input = remaining,
            Err(nom::Err::Error(e)) => {
                return Err(nom::Err::Error(
                    E::append(input, error::ErrorKind::Count, e).with_cause(
                        error::ErrorCause::Vector(InvalidVector::Remaining {
                            expected: (count - i).try_into().unwrap_or(u32::MAX),
                        }),
                    ),
                ))
            }
            Err(err) => return Err(err),
        }
    }

    Ok((input, ()))
}

#[inline]
pub(crate) fn sequence<'a, E, C, P>(input: &'a [u8], count: C, parser: P) -> Parsed<'a, (), E>
where
    E: ErrorSource<'a>,
    C: nom::ToUsize,
    P: nom::Parser<&'a [u8], (), E>,
{
    sequence_inner(input, count.to_usize(), parser)
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
