//! Functions for parsing integers in the
//! [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).

use crate::{
    error::{ErrorCause, ErrorKind, ErrorSource},
    Parsed,
};

const MORE_FLAG: u8 = 0b1000_0000;
const SIGN_FLAG: u8 = 0b0100_0000;
const VALUE_MASK: u8 = 0b0111_1111;

/// Represents the target type for a particular *LEB128* encoded integer value.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Destination {
    U32,
    S32,
    U64,
    S64,
}

/// Describes why a *LEB128* integer count not be decoded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InvalidEncoding {
    /// The encoded value would result in an overflow when converted to the target type.
    Overflow,
    /// More bytes containing value bits were expected.
    NoContinuation,
}

macro_rules! unsigned {
    ($(
        $(#[$meta:meta])*
        $integer:ty => $name:ident[$destination:ident];
    )*) => {$(
        $(#[$meta])*
        pub fn $name<'a, E: ErrorSource<'a>>(mut input: &'a [u8]) -> Parsed<'a, $integer, E> {
            let start = input;
            let mut result: $integer = 0;
            let mut shift = 0usize;
            loop {
                if let Some((byte, remaining)) = input.split_first() {
                    input = remaining;

                    // TODO: Use CLZ?
                    let valid_mask = !(0xFFu8 << (<$integer>::BITS as usize - shift).min(7));
                    if byte & !(MORE_FLAG | valid_mask) != 0 {
                        return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                            start,
                            ErrorKind::TooLarge,
                            ErrorCause::Leb128 {
                                destination: Destination::$destination,
                                reason: InvalidEncoding::Overflow,
                            },
                        )));
                    }

                    result |= (((byte & valid_mask) as $integer) << shift);
                    shift += 7;

                    if byte & MORE_FLAG == 0 {
                        break;
                    }
                } else {
                    return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                        start,
                        ErrorKind::Complete,
                        ErrorCause::Leb128 {
                            destination: Destination::$destination,
                            reason: InvalidEncoding::Overflow,
                        },
                    )));
                }
            }

            return Ok((input, result))
        }
    )*};
}

unsigned! {
    /// Parses an at most 5-byte wide *LEB128* encoded unsigned 32-bit integer.
    u32 => u32[U32];
    /// Parses an at most 10-byte wide *LEB128* encoded unsigned 64-bit integer.
    u64 => u64[U64];
}
