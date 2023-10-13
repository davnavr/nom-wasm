//! Parsers recognizing common structures described in the WebAssembly binary format
//! [conventions] and [values] specification.
//!
//! [conventions]: https://webassembly.github.io/spec/core/binary/conventions.html
//! [values]: https://webassembly.github.io/spec/core/binary/values.html

use crate::error::{AddCause as _, ErrorCause, ErrorKind, ErrorSource};
use nom::ToUsize;

mod vector;

pub mod leb128;

pub(crate) use vector::sequence;

pub use leb128::{s32 as leb128_s32, s64 as leb128_s64, u32 as leb128_u32, u64 as leb128_u64};
pub use vector::{vector, vector_length, VectorIter};

/// Parses a [WebAssembly **`name`**] prefixed by a [*LEB128* length] from the given `input`.
///
/// [WebAssembly **`name`**]: https://webassembly.github.io/spec/core/binary/values.html#names
/// [*LEB128* length]: leb128_u32
pub fn name<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> crate::Parsed<'a, &'a str, E> {
    let (input, length) = leb128_u32(input).add_cause(ErrorCause::SectionLength)?;

    if let Some(contents) = input.get(..length.to_usize()) {
        match core::str::from_utf8(contents) {
            Ok(name) => Ok((&input[length.to_usize()..], name)),
            Err(err) => Err(nom::Err::Failure(E::from_error_kind_and_cause(
                contents,
                ErrorKind::Verify,
                ErrorCause::NameEncoding(err),
            ))),
        }
    } else {
        Err(nom::Err::Failure(E::from_error_kind_and_cause(
            input,
            ErrorKind::Eof,
            ErrorCause::NameContents(crate::error::LengthMismatch {
                expected: length,
                actual: input.len().try_into().unwrap_or(u32::MAX),
            }),
        )))
    }
}
