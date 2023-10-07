//! Functions for parsing integers in the
//! [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).

use crate::{
    error::{ErrorCause, ErrorKind, ErrorSource},
    Parsed,
};

const MORE_FLAG: u8 = 0b1000_0000;
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

macro_rules! unsigned_parsers {
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
                            reason: InvalidEncoding::NoContinuation,
                        },
                    )));
                }
            }

            return Ok((input, result))
        }
    )*};
}

macro_rules! signed_parsers {
    ($(
        $(#[$meta:meta])*
        $integer:ident ^ $storage:ident => $name:ident[$destination:ident];
    )*) => {$(
        $(#[$meta])*
        pub fn $name<'a, E: ErrorSource<'a>>(mut input: &'a [u8]) -> Parsed<'a, $integer, E> {
            const SIGN_FLAG: u8 = 0b0100_0000;
            const STORAGE_BITS: usize = <$storage>::BITS as usize;

            let start = input;
            let mut destination: $storage = 0;
            let mut shift = 0usize;
            loop {
                if let Some((byte, remaining)) = input.split_first() {
                    input = remaining;

                    destination |= ((byte & VALUE_MASK) as $storage) << shift;
                    shift += 7;

                    if byte & MORE_FLAG == 0 {
                        // Sign extension
                        destination |= (((byte & SIGN_FLAG) as $storage) << (STORAGE_BITS - 7)) >> (STORAGE_BITS - shift - 1);
                        break;
                    }
                } else {
                    return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                        start,
                        ErrorKind::Complete,
                        ErrorCause::Leb128 {
                            destination: Destination::$destination,
                            reason: InvalidEncoding::NoContinuation,
                        },
                    )));
                }
            }

            if let Ok(result) = $integer::try_from(destination) {
                Ok((input, result))
            } else {
                Err(nom::Err::Failure(E::from_error_kind_and_cause(
                    start,
                    ErrorKind::TooLarge,
                    ErrorCause::Leb128 {
                        destination: Destination::$destination,
                        reason: InvalidEncoding::Overflow,
                    },
                )))
            }
        }
    )*};
}

unsigned_parsers! {
    /// Parses an at most 5-byte wide *LEB128* encoded unsigned 32-bit integer.
    u32 => u32[U32];
    /// Parses an at most 10-byte wide *LEB128* encoded unsigned 64-bit integer.
    u64 => u64[U64];
}

signed_parsers! {
    /// Parses an at most 5-byte wide *LEB128* encoded signed 32-bit integer.
    i32 ^ i64 => s32[S32];
    /// Parses an at most 10-byte wide *LEB128* encoded signed 64-bit integer.
    i64 ^ i128 => s64[S64];
}
