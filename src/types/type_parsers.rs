use crate::{
    error::{AddCause, ErrorCause, ErrorKind, ErrorSource},
    types::{BlockType, ValType},
    values::leb128,
    Parsed,
};
use nom::Parser;

impl BlockType {
    /// Parses a [`BlockType`].
    ///
    /// # Error
    ///
    /// Returns an error if an unrecognized type was encountered, or an encoded 33-bit type index is greater than the maximum
    /// value for 32-bit indices.
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        let start = input;
        let (input, value) = leb128::s64(input).add_cause(ErrorCause::BlockType(None))?;

        let block_type = match value {
            -64 => Self::Empty,
            -1 => Self::Inline(ValType::I32),
            -2 => Self::Inline(ValType::I64),
            -3 => Self::Inline(ValType::F32),
            -4 => Self::Inline(ValType::F64),
            -5 => Self::Inline(ValType::V128),
            -16 => Self::Inline(ValType::FuncRef),
            -17 => Self::Inline(ValType::ExternRef),
            _ if value < 0 => {
                // Unknown
                return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                    start,
                    ErrorKind::Tag,
                    ErrorCause::BlockType(core::num::NonZeroI64::new(value)),
                )));
            }
            _ => {
                if let Ok(index) = u32::try_from(value) {
                    Self::Index(index.into())
                } else {
                    debug_assert!(value != 0);

                    // Type index too large
                    return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                        start,
                        ErrorKind::Verify,
                        ErrorCause::BlockType(core::num::NonZeroI64::new(value)),
                    )));
                }
            }
        };

        Ok((input, block_type))
    }
}

impl ValType {
    /// Parses a [`ValType`].
    ///
    /// See [`BlockType::parse()`] for more information.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`BlockType`] could not be parsed, or if a [`BlockType::Empty`] or
    /// [`BlockType::Index`] was parsed.
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        match BlockType::parse(input)? {
            (input, BlockType::Inline(val_type)) => Ok((input, val_type)),
            (_, BlockType::Empty) => Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Verify,
                ErrorCause::ValType(None),
            ))),
            (_, BlockType::Index(index)) => Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Verify,
                ErrorCause::ValType(Some(index)),
            ))),
        }
    }
}

/// Provides an explicit [`Parser`] implementation for [`ValType::parse()`].
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct ValTypeParser;

impl<'a, E: ErrorSource<'a>> Parser<&'a [u8], ValType, E> for ValTypeParser {
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> nom::IResult<&'a [u8], ValType, E> {
        ValType::parse(input)
    }
}
