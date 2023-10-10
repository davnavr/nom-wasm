//! Contains types describing errors that occur during parsing.

mod add_cause;
mod cause;

#[cfg(feature = "alloc")]
mod verbose_error;

pub use cause::{
    ErrorCause, ImportComponent, InvalidFlags, InvalidFlagsValue, InvalidTag, LengthMismatch,
    LimitsComponent,
};
#[doc(no_inline)]
pub use nom::error::ErrorKind;
#[cfg(feature = "alloc")]
pub use verbose_error::VerboseError;

pub(crate) use add_cause::AddCause;

/// Default error type, which tracks an error's location, the kind of error that occured, and why
/// it occured.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Error<'a> {
    /// A slice into the original input indicating where the error occured.
    pub input: &'a [u8],
    /// The [`nom`] error code describing the kind of error that occured.
    pub kind: ErrorKind,
    /// A WASM parsing specific error code indicating why a parse failed.
    pub cause: Option<ErrorCause>,
}

impl<'a> From<nom::error::Error<&'a [u8]>> for Error<'a> {
    #[inline]
    fn from(error: nom::error::Error<&'a [u8]>) -> Self {
        Self {
            input: error.input,
            kind: error.code,
            cause: None,
        }
    }
}

impl<'a> nom::error::ParseError<&'a [u8]> for Error<'a> {
    #[inline]
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Self {
            input,
            kind,
            cause: None,
        }
    }

    #[inline]
    fn append(_: &'a [u8], _: ErrorKind, other: Self) -> Self {
        other
    }
}

//#[cfg_attr(doc_cfg, doc(cfg(feature = "error_stack")))]
//#[cfg(feature = "std")]
//impl std::error::Error for Error<'_> {}

/// Common trait bounds required for an error type to be used by [`nom-wasm`](crate).
pub trait ErrorSource<'a>: nom::error::ParseError<&'a [u8]> {
    /// Attaches the given [`ErrorCause`] to an existing error.
    #[inline]
    fn with_cause(self, cause: ErrorCause) -> Self {
        let _ = cause;
        self
    }

    /// Creates a new error from the input where it occured, the `kind` of error that occured, and
    /// an [`ErrorCause`].
    #[inline]
    fn from_error_kind_and_cause(input: &'a [u8], kind: ErrorKind, cause: ErrorCause) -> Self {
        Self::from_error_kind(input, kind).with_cause(cause)
    }
}

impl ErrorSource<'_> for () {}

impl<'a> ErrorSource<'a> for (&'a [u8], ErrorKind) {}

impl<'a> ErrorSource<'a> for nom::error::Error<&'a [u8]> {}

impl<'a> ErrorSource<'a> for Error<'a> {
    #[inline]
    fn with_cause(mut self, cause: ErrorCause) -> Self {
        self.cause = Some(cause);
        self
    }

    #[inline]
    fn from_error_kind_and_cause(input: &'a [u8], kind: ErrorKind, cause: ErrorCause) -> Self {
        Self {
            input,
            kind,
            cause: Some(cause),
        }
    }
}
