//! Functions for parsing integers in the
//! [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).

use crate::parser::{ParseFailed, Parsed};
use nom::error::ErrorKind;

const MORE_FLAG: u8 = 0b1000_0000;
const SIGN_FLAG: u8 = 0b0100_0000;
const VALUE_MASK: u8 = 0b0111_1111;

macro_rules! unsigned {
    ($(
        $(#[$meta:meta])*
        $integer:ty => $name:ident;
    )*) => {$(
        $(#[$meta])*
        pub fn $name<'a, E: ParseFailed<'a>>(mut input: &'a [u8]) -> Parsed<'a, $integer, E> {
            let start = input;
            let mut result: $integer = 0;
            let mut shift = 0usize;
            loop {
                if let Some((byte, remaining)) = input.split_first() {
                    input = remaining;

                    let valid_mask = !(0xFFu8 << (<$integer>::BITS as usize - shift).min(7));
                    if byte & !(MORE_FLAG | valid_mask) != 0 {
                        return Err(nom::Err::Failure(E::add_context(
                            start,
                            "encoded LEB128 unsigned integer value would overflow",
                            E::from_error_kind(start, ErrorKind::TooLarge)
                        )));
                    }

                    result |= (((byte & valid_mask) as $integer) << shift);
                    shift += 7;

                    if byte & MORE_FLAG == 0 {
                        break;
                    }
                } else {
                    return Err(nom::Err::Failure(E::from_error_kind(start, ErrorKind::Complete)));
                }
            }

            return Ok((input, result))
        }
    )*};
}

unsigned! {
    /// Parses an at most 5-byte wide *LEB128* encoded unsigned 32-bit integer.
    u32 => u32;
    /// Parses an at most 10-byte wide *LEB128* encoded unsigned 64-bit integer.
    u64 => u64;
}
