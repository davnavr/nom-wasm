use crate::{
    error::{AddCause, ErrorCause, ErrorKind, ErrorSource},
    leb128,
    types::{self, BlockType, ValType},
    Parsed,
};

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
