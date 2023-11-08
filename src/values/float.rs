use crate::{
    error::{ErrorKind, ErrorSource},
    Parsed,
};
use core::fmt::{Debug, Formatter};

/// Contains the WebAssembly binary format encoding for an IEEE-754 `binary32`, which encodes an
/// [`f32`] in little-endian byte order.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct F32(pub [u8; 4]);

/// Contains the WebAssembly binary format encoding for an IEEE-754 `binary64`, which encodes an
/// [`f64`] in little-endian byte order.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct F64(pub [u8; 8]);

#[allow(missing_docs)]
impl F32 {
    pub fn interpret(self) -> f32 {
        f32::from_le_bytes(self.0)
    }

    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, F32, E> {
        if let Some(bytes) = input.get(..4) {
            Ok((&input[4..], Self(bytes.try_into().unwrap())))
        } else {
            Err(nom::Err::Failure(E::from_error_kind(input, ErrorKind::Eof)))
        }
    }
}

#[allow(missing_docs)]
impl F64 {
    pub fn interpret(self) -> f64 {
        f64::from_le_bytes(self.0)
    }

    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, F64, E> {
        if let Some(bytes) = input.get(..8) {
            Ok((&input[8..], Self(bytes.try_into().unwrap())))
        } else {
            Err(nom::Err::Failure(E::from_error_kind(input, ErrorKind::Eof)))
        }
    }
}

impl Debug for F32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#010X}", u32::from_le_bytes(self.0))
    }
}

impl Debug for F64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018X}", u64::from_le_bytes(self.0))
    }
}
