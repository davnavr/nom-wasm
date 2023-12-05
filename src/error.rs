//! Contains types describing errors that occur during parsing.

mod add_cause;
mod cause;
mod component;
mod invalid_flags;
mod invalid_tag;
mod length_mismatch;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
mod verbose_error;

#[doc(no_inline)]
pub use crate::{
    isa::{InvalidExpr, InvalidInstr, InvalidOpcode},
    module::preamble::InvalidMagic,
    values::InvalidVector,
};

pub use cause::ErrorCause;
pub use component::{ImportComponent, LimitsComponent, MemArgComponent};
pub use invalid_flags::{InvalidFlags, InvalidFlagsValue};
pub use invalid_tag::InvalidTag;
pub use length_mismatch::LengthMismatch;

#[cfg(feature = "alloc")]
pub use verbose_error::VerboseError;

pub(crate) use add_cause::AddCause;

/// Default error type, which tracks an error's location and the reason why it occured.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct Error<'a> {
    /// A slice into the original input indicating where the error occured.
    pub input: &'a [u8],
    /// An error code indicating why a parse failed.
    pub cause: ErrorCause,
}

impl core::fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Error")
            .field("input", &crate::hex::Bytes(self.input))
            .field("cause", &self.cause)
            .finish()
    }
}

impl<'a> From<nom::error::Error<&'a [u8]>> for Error<'a> {
    #[inline]
    fn from(error: nom::error::Error<&'a [u8]>) -> Self {
        Self {
            input: error.input,
            cause: ErrorCause::Nom(error.code),
        }
    }
}

impl<'a> nom::error::ParseError<&'a [u8]> for Error<'a> {
    #[inline]
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        Self {
            input,
            cause: ErrorCause::Nom(kind),
        }
    }

    #[inline]
    fn append(_: &'a [u8], _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl core::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.cause, f)?;

        if !self.input.is_empty() {
            write!(f, ", for {} bytes in input:", self.input.len())?;

            const DISPLAY_MAX: usize = 8;

            for b in self.input.iter().take(DISPLAY_MAX) {
                write!(f, " {b:02X}")?;
            }

            if self.input.len() > DISPLAY_MAX {
                f.write_str(" ...")?;
            }
        }

        Ok(())
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for Error<'_> {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        std::error::Error::source(&self.cause)
    }
}

/// Trait for error types used with [`nom-wasm`](crate).
pub trait ErrorSource<'a>: nom::error::ParseError<&'a [u8]> {
    /// Combines existing error with a newly constructed error.
    #[inline]
    fn append_with_cause(input: &'a [u8], cause: ErrorCause, other: Self) -> Self {
        let _ = (input, cause);
        other
    }

    /// Creates a new error from the input where it occured, and its [`ErrorCause`].
    #[inline]
    fn from_error_cause(input: &'a [u8], cause: ErrorCause) -> Self {
        Self::from_error_kind(input, cause.to_error_kind())
    }
}

impl ErrorSource<'_> for () {}

impl<'a> ErrorSource<'a> for (&'a [u8], nom::error::ErrorKind) {}

impl<'a> ErrorSource<'a> for nom::error::Error<&'a [u8]> {}

impl<'a> ErrorSource<'a> for nom::error::VerboseError<&'a [u8]> {}

impl<'a> ErrorSource<'a> for Error<'a> {
    #[inline]
    fn from_error_cause(input: &'a [u8], cause: ErrorCause) -> Self {
        Self { input, cause }
    }
}
