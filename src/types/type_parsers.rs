use crate::{
    error::{self, AddCause, ErrorCause, ErrorKind, ErrorSource},
    types::{self, BlockType, Limits, ValType},
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
    /// See the documentation for [`BlockType::parse()`] for more information.
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

impl types::RefType {
    /// Parses a [`RefType`](types::RefType).
    ///
    /// See the documentation for [`ValType::parse()`] for more information.
    ///
    /// # Errors
    ///
    /// Returns an error if some other [`ValType`] is parsed instead, or if the type was not
    /// encoded correctly.
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        match ValType::parse(input)? {
            (input, ValType::FuncRef) => Ok((input, Self::Func)),
            (input, ValType::ExternRef) => Ok((input, Self::Extern)),
            (_, bad) => Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Verify,
                ErrorCause::RefType(bad),
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

impl Limits {
    #[allow(missing_docs)]
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        let (input, flags) = if let Some((first, input)) = input.split_first() {
            (input, *first)
        } else {
            return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::OneOf,
                ErrorCause::InvalidFlags(error::InvalidFlags::Limits(
                    error::InvalidFlagsValue::Missing,
                )),
            )));
        };

        const VALID_MASK: u8 = 0b111;

        let invalid = flags & (!VALID_MASK);
        if invalid != 0 {
            return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                &input[..1],
                ErrorKind::Verify,
                ErrorCause::InvalidFlags(error::InvalidFlags::Limits(
                    error::InvalidFlagsValue::Invalid {
                        value: flags,
                        invalid,
                    },
                )),
            )));
        }

        const USE_MEMORY_64: u8 = 0b100;
        const HAS_MAXIMUM: u8 = 1;

        let has_maximum = flags & HAS_MAXIMUM != 0;

        macro_rules! parse_bounds {
            ($parser:ident => $idx:ident) => {{
                let index_type = types::IdxType::$idx;

                let (input, min) = leb128::$parser(input).add_cause(ErrorCause::Limits {
                    index_type,
                    component: error::LimitsComponent::Minimum,
                })?;

                let (input, max) = if has_maximum {
                    leb128::$parser(input)
                        .add_cause(ErrorCause::Limits {
                            index_type,
                            component: error::LimitsComponent::Maximum,
                        })
                        .map(|(input, max)| (input, Some(max)))?
                } else {
                    (input, None)
                };

                (input, types::LimitBounds::$idx { min, max })
            }};
        }

        let (input, bounds) = if flags & USE_MEMORY_64 == 0 {
            parse_bounds!(u32 => I32)
        } else {
            parse_bounds!(u64 => I64) // memory64
        };

        const IS_SHARED: u8 = 0b10;

        let share = if flags & IS_SHARED == 0 {
            types::Sharing::Unshared
        } else {
            types::Sharing::Shared
        };

        Ok((input, Self { bounds, share }))
    }
}

impl types::GlobalType {
    #[allow(missing_docs)]
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        let (input, value_type) = ValType::parse(input).add_cause(ErrorCause::GlobalType)?;

        let (input, flags) = if let Some((first, input)) = input.split_first() {
            (input, *first)
        } else {
            return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::OneOf,
                ErrorCause::InvalidFlags(error::InvalidFlags::GlobalType(
                    error::InvalidFlagsValue::Missing,
                )),
            )));
        };

        let mutability = match flags {
            0 => types::Mutability::Constant,
            1 => types::Mutability::Variable,
            _ => {
                return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                    &input[..1],
                    ErrorKind::OneOf,
                    ErrorCause::InvalidFlags(error::InvalidFlags::GlobalType(
                        error::InvalidFlagsValue::Invalid {
                            value: flags,
                            invalid: flags & (!1u8),
                        },
                    )),
                )))
            }
        };

        Ok((
            input,
            Self {
                mutability,
                value_type,
            },
        ))
    }
}

impl types::MemType {
    /// Parses a [`MemType`](types::MemType).
    ///
    /// See the documentation for [`Limits::parse()`] for more information.
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        Limits::parse(input)
            .add_cause(ErrorCause::MemType)
            .map(|(input, limits)| (input, Self { limits }))
    }
}

impl types::TableType {
    #[allow(missing_docs)]
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        let (input, element_type) =
            types::RefType::parse(input).add_cause(ErrorCause::TableType)?;

        let (input, limits) = Limits::parse(input).add_cause(ErrorCause::TableType)?;

        Ok((
            input,
            Self {
                element_type,
                limits,
            },
        ))
    }
}

impl types::TagType {
    #[allow(missing_docs)]
    pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        let (input, _) = nom::bytes::complete::tag(&[0u8])(input).add_cause(ErrorCause::TagType)?;
        let (input, index) = crate::index::Index::parse(input).add_cause(ErrorCause::TagType)?;
        Ok((input, Self::Exception(index)))
    }
}
