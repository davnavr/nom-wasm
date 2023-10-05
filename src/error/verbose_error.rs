use crate::error;
use alloc::boxed::Box;
use core::fmt::Debug;
use nom::error::ParseError;

#[derive(PartialEq)]
enum Error<'a> {
    Error(nom::error::Error<&'a [u8]>),
    Cause(error::ErrorCause),
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Error(e) => Debug::fmt(e, f),
            Self::Cause(c) => Debug::fmt(c, f),
        }
    }
}

#[derive(PartialEq)]
struct Inner<'a> {
    base: nom::error::Error<&'a [u8]>,
    additional: alloc::vec::Vec<Error<'a>>,
}

/// Accumulates information about an error, including its location, the kinds of error that
/// occured, and the reasons why it occured.
#[derive(PartialEq)]
#[repr(transparent)]
pub struct VerboseError<'a> {
    inner: Box<Inner<'a>>,
}

impl Debug for VerboseError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entry(&self.inner.base)
            .entries(&self.inner.additional)
            .finish()
    }
}

impl<'a> ParseError<&'a [u8]> for VerboseError<'a> {
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        Self {
            inner: Box::new(Inner {
                base: ParseError::from_error_kind(input, kind),
                additional: Default::default(),
            }),
        }
    }

    fn append(input: &'a [u8], kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other
            .inner
            .additional
            .push(Error::Error(ParseError::from_error_kind(input, kind)));

        other
    }
}

impl<'a> error::ErrorSource<'a> for VerboseError<'a> {
    fn from_error_kind_and_cause(
        input: &'a [u8],
        kind: nom::error::ErrorKind,
        cause: error::ErrorCause,
    ) -> Self {
        Self {
            inner: Box::new(Inner {
                base: ParseError::from_error_kind(input, kind),
                additional: vec![Error::Cause(cause)],
            }),
        }
    }

    fn with_cause(mut self, cause: error::ErrorCause) -> Self {
        self.inner.additional.push(Error::Cause(cause));
        self
    }
}
