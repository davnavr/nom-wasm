use crate::{error::ErrorSource, Parsed};
use core::fmt::{Debug, Formatter};

fn parse_16_bytes<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, [u8; 16], E> {
    if let Some(bytes) = input.get(..16) {
        Ok((&input[16..], bytes.try_into().unwrap()))
    } else {
        Err(nom::Err::Failure(E::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        )))
    }
}

/// Represents a `v128` WebAssembly value.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct V128(pub [u8; 16]);

impl V128 {
    #[allow(missing_docs)]
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        nom::combinator::map(parse_16_bytes, Self)(input)
    }
}

impl Debug for V128 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#034X}", u128::from_le_bytes(self.0))
    }
}

/// Represents a list of indices for a 128-bit vector value used in "shuffle" operations.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct V128ShuffleLanes(pub [crate::isa::LaneIdx; 16]);

impl V128ShuffleLanes {
    #[inline]
    #[allow(missing_docs)]
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        nom::combinator::map(parse_16_bytes, Self)(input)
    }
}

impl Debug for V128ShuffleLanes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.0).finish()
    }
}
