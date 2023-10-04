use crate::leb128;

/// Describes why a parser error occured.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorCause {
    #[allow(missing_docs)]
    PreambleMagic(arrayvec::ArrayVec<u8, 4>),
    #[allow(missing_docs)]
    PreambleVersion(Option<u32>),
    #[allow(missing_docs)]
    Leb128 {
        destination: leb128::Destination,
        reason: leb128::InvalidEncoding,
    },
    #[allow(missing_docs)]
    SectionId,
    #[allow(missing_docs)]
    SectionLength,
    #[allow(missing_docs)]
    SectionContents { expected: u32, actual: u32 },
}

impl core::fmt::Display for ErrorCause {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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
            Self::SectionId => f.write_str("missing section ID byte"),
            Self::SectionLength => f.write_str("missing section content length"),
            Self::SectionContents { expected, actual } => write!(f, "expected {expected} bytes of section contents, but there were {actual} bytes remaining"),
        }
    }
}
