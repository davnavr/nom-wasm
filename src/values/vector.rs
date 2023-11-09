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

#[deprecated]
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

#[deprecated]
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
#[deprecated]
pub fn vector<'a, E, P>(input: &'a [u8], parser: P) -> Parsed<'a, (), E>
where
    E: ErrorSource<'a>,
    P: nom::Parser<&'a [u8], (), E>,
{
    let (input, count) = vector_length(input)?;
    sequence(input, count, parser)
}

fn sequence_fold_inner<'a, O, E, R>(
    count: usize,
    mut init: impl FnMut() -> R,
    mut parser: impl Parser<&'a [u8], O, E>,
    mut fold: impl FnMut(usize, R, O) -> R,
) -> impl FnMut(&'a [u8]) -> Parsed<'a, R, E>
where
    E: ErrorSource<'a>,
{
    move |mut input| {
        let mut state = init();
        for i in 0..count {
            match parser.parse(input) {
                Ok((remaining, item)) => {
                    state = fold(i, state, item);
                    input = remaining;
                }
                Err(err) => {
                    return Err(err.map(|other| {
                        let expected = (count - i).try_into().unwrap_or(u32::MAX);
                        E::append(input, error::ErrorKind::Count, other).with_cause(
                            error::ErrorCause::Vector(InvalidVector::Remaining { expected }),
                        )
                    }))
                }
            }
        }

        Ok((input, state))
    }
}

pub(crate) fn sequence_fold<'a, O, E, R, C, I, P, F>(
    count: C,
    init: I,
    parser: P,
    fold: F,
) -> impl Parser<&'a [u8], R, E>
where
    E: ErrorSource<'a>,
    I: FnMut() -> R,
    P: Parser<&'a [u8], O, E>,
    F: FnMut(usize, R, O) -> R,
    C: nom::ToUsize,
{
    sequence_fold_inner(count.to_usize(), init, parser, fold)
}

/// Parses a [WebAssembly vector], which is a [`u32` length] followed by elements parsed by the
/// given `parser`.
///
/// [WebAssembly vector]: https://webassembly.github.io/spec/core/binary/conventions.html#vectors
/// [`u32` length]: vector_length
pub fn vector_fold<'a, O, E, R, I, P, F>(
    mut init: I,
    mut parser: P,
    fold: F,
) -> impl Parser<&'a [u8], R, E>
where
    E: ErrorSource<'a>,
    I: FnMut(usize) -> R,
    P: Parser<&'a [u8], O, E>,
    F: FnMut(usize, R, O) -> R,
{
    // See https://users.rust-lang.org/t/help-with-nom-error/101613/3
    // nom should have G: FnMut

    // vector_length.flat_map(|count| {
    //     sequence_fold(
    //         count,
    //         || init(nom::ToUsize::to_usize(&count)),
    //         |input| parser.parse(input),
    //         fold,
    //     )
    // })
    struct VectorFold<I, P, F, O> {
        init: I,
        parser: P,
        fold: F,
        _marker: core::marker::PhantomData<fn() -> O>,
    }

    impl<'a, O, E, R, I, P, F> Parser<&'a [u8], R, E> for VectorFold<I, P, F, O>
    where
        E: ErrorSource<'a>,
        I: FnMut(usize) -> R,
        P: Parser<&'a [u8], O, E>,
        F: FnMut(usize, R, O) -> R,
    {
        fn parse(&mut self, input: &'a [u8]) -> Parsed<'a, R, E> {
            let (input, count) = vector_length(input)?;
            let mut parse_elements = sequence_fold(
                count,
                || (self.init)(nom::ToUsize::to_usize(&count)),
                |input| self.parser.parse(input),
                &mut self.fold,
            );
            parse_elements.parse(input)
        }
    }

    VectorFold {
        init,
        parser,
        fold,
        _marker: core::marker::PhantomData,
    }
}

pub fn vector_collect<'a, O, E, I, C, P>(init: I, parser: P) -> impl Parser<&'a [u8], C, E>
where
    E: ErrorSource<'a>,
    I: FnMut(usize) -> C,
    C: Extend<O>,
    P: Parser<&'a [u8], O, E>,
{
    vector_fold(init, parser, |_, mut collection, item| {
        collection.extend(std::iter::once(item));
        collection
    })
}

//pub fn vector2<'a, O, E, P>(parser: P) -> impl Parser<&'a [u8], alloc::vec::Vec<O>, E> {
//
//}
