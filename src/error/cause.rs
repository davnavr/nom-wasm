use crate::leb128;
use core::fmt::{Display, Formatter};

/// Describes an [`ErrorCause`] where the length of some data was incorrect.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub struct LengthMismatch {
    pub expected: u32,
    pub actual: u32,
}

impl LengthMismatch {
    fn print(&self, name: &str, f: &mut Formatter) -> core::fmt::Result {
        write!(
            f,
            "expected {} bytes for {name}, but got {}",
            self.expected, self.actual
        )
    }
}

/// Error type used when a byte or 32-bit enumeration value was invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum InvalidTag {
    /// An invalid [`ModuleSectionId`](crate::module::ModuleSectionId).
    ModuleSectionId(u8),
}

impl Display for InvalidTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let (value, value_width) = match self {
            Self::ModuleSectionId(b) => (u32::from(*b), 4),
        };

        let name = match self {
            Self::ModuleSectionId(_) => "module section ID",
        };

        write!(
            f,
            "the {name} tag {value:#0value_width$X} ({value}) are invalid"
        )
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidTag {}

/// Describes why a parser error occured.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ErrorCause {
    Leb128 {
        destination: leb128::Destination,
        reason: leb128::InvalidEncoding,
    },
    InvalidTag(InvalidTag),
    #[non_exhaustive]
    NameLength,
    NameContents(LengthMismatch),
    NameEncoding(core::str::Utf8Error),
    #[non_exhaustive]
    SectionId,
    #[non_exhaustive]
    SectionLength,
    SectionContents(LengthMismatch),
    #[non_exhaustive]
    CustomSectionName,
    PreambleMagic(arrayvec::ArrayVec<u8, 4>),
    PreambleVersion(Option<u32>),
    /// A [`BlockType`](crate::types::BlockType) could not be parsed.
    /// - Contains `None` if the end of input was unexpectedly encountered.
    /// - Contains `Some` negative value if an unrecognized encoding for a type was encountered.
    /// - Contains `Some` positive value if the parsed [`typeidx`] was too large.
    ///
    /// [`typeidx`]: crate::module::TypeIdx
    BlockType(Option<core::num::NonZeroI64>),
    /// A [`ValType`](crate::types::ValType) was not valid.
    ///
    /// - Contains `None` if a [`BlockType::Empty`] was parsed.
    /// - Contains `Some` type index if a [`BlockType::Index`] was parsed.
    ///
    /// [`BlockType::Empty`]: crate::types::BlockType::Empty
    /// [`BlockType::Index`]: crate::types::BlockType::Index
    ValType(Option<crate::module::TypeIdx>),
}

const _SIZE_CHECK: () = if core::mem::size_of::<ErrorCause>() > 16 {
    panic!("ErrorCause is too large")
};

impl Display for ErrorCause {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Leb128 {
                destination,
                reason,
            } => {
                f.write_str("could not parse LEB128 encoded ")?;

                match destination {
                    leb128::Destination::U32 | leb128::Destination::U64 => f.write_str("un")?,
                    leb128::Destination::S32 | leb128::Destination::S64 => (),
                }

                f.write_str("signed ")?;

                match destination {
                    leb128::Destination::U32 | leb128::Destination::S32 => f.write_str("32")?,
                    leb128::Destination::U64 | leb128::Destination::S64 => f.write_str("64")?,
                }

                f.write_str("-bit integer")?;

                match reason {
                    leb128::InvalidEncoding::Overflow => {
                        write!(f, ", an overflow occured while decoding the value")
                    }
                    leb128::InvalidEncoding::NoContinuation => Ok(()),
                }
            }
            Self::InvalidTag(tag) => Display::fmt(tag, f),
            Self::NameLength => f.write_str("expected name length"),
            Self::NameContents(e) => e.print("UTF-8 encoded name", f),
            Self::NameEncoding(e) => write!(f, "invalid name encoding: {e}"),
            Self::SectionId => f.write_str("missing section ID byte"),
            Self::SectionLength => f.write_str("expected section content length"),
            Self::SectionContents(e) => e.print("section contents", f),
            Self::CustomSectionName => f.write_str("expected custom section name"),
            Self::PreambleMagic(actual) => {
                f.write_str("not a valid WASM module, ")?;

                f.write_str(if actual.is_empty() {
                    "missing"
                } else {
                    "expected"
                })?;

                write!(
                    f,
                    " WebAssembly magic \"{}\"",
                    crate::module::preamble::MAGIC.escape_ascii()
                )?;

                if !actual.is_empty() {
                    write!(f, ", but got {}", actual.escape_ascii())?;
                }

                Ok(())
            }
            Self::PreambleVersion(None) => f.write_str("missing WASM preamble version"),
            Self::PreambleVersion(Some(actual)) => {
                let expected = u32::from_le_bytes(crate::module::preamble::RECOGNIZED_VERSION);
                write!(f, "expected WASM preamble version {expected} ({expected:#010X}), but got {actual} ({actual:#010X})")
            }
            Self::BlockType(None) => f.write_str("expected valtype, typeidx, or empty block type"),
            Self::BlockType(Some(block_type)) => {
                if block_type.get() < 0 {
                    write!(f, "{block_type} is not a valid value type or block type")
                } else {
                    write!(f, "type index in block type {block_type} is too large, maximum 32-bit value is {}", u32::MAX)
                }
            }
            Self::ValType(None) => f.write_str("expected valtype but got empty block type"),
            Self::ValType(Some(index)) => write!(f, "expected valtype but got type index {index}"),
        }
    }
}
