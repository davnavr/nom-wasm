use crate::{
    error::{self, AddCause as _, ErrorSource},
    Parsed,
};

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

/// Parses a [WebAssembly `vec`tor], which is a sequence of elements prefixed by a [`u32` length].
///
/// [WebAssembly `vec`tor]: https://webassembly.github.io/spec/core/binary/conventions.html#vectors
/// [`u32` length]: vector_length
pub fn vector<'a, E, P>(input: &'a [u8], parser: P) -> Parsed<'a, (), E>
where
    E: ErrorSource<'a>,
    P: nom::Parser<&'a [u8], (), E>,
{
    let (input, count) = vector_length(input)?;
    sequence(input, count, parser)
}
