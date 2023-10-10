//! Utilities for [`Debug`] formatting.

use crate::error::Error;
use core::fmt::{Debug, Formatter};

// TODO: Deprecate this, use ParseDebug instead
pub(crate) struct FmtResult<T, E>(pub(crate) core::result::Result<T, E>);

impl<T: Debug, E: Debug> Debug for FmtResult<T, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let item: &dyn Debug = match &self.0 {
            Ok(ok) => ok,
            Err(err) => err,
        };

        item.fmt(f)
    }
}

pub(crate) enum ParseFailedState<'a, 'b: 'a> {
    List(core::fmt::DebugList<'a, 'b>),
    Struct(core::fmt::DebugStruct<'a, 'b>),
}

impl<'a, 'b: 'a> From<core::fmt::DebugList<'a, 'b>> for ParseFailedState<'a, 'b> {
    #[inline]
    fn from(debug: core::fmt::DebugList<'a, 'b>) -> Self {
        Self::List(debug)
    }
}

impl<'a, 'b: 'a> From<core::fmt::DebugStruct<'a, 'b>> for ParseFailedState<'a, 'b> {
    #[inline]
    fn from(debug: core::fmt::DebugStruct<'a, 'b>) -> Self {
        Self::Struct(debug)
    }
}

pub(crate) struct ParseFailed<'i, 'a, 'b: 'a> {
    state: ParseFailedState<'a, 'b>,
    error: nom::Err<Error<'i>>,
}

impl<'i, 'a, 'b: 'a> ParseFailed<'i, 'a, 'b> {
    pub(crate) fn new(
        state: impl Into<ParseFailedState<'a, 'b>>,
        error: nom::Err<Error<'i>>,
    ) -> Self {
        Self {
            state: state.into(),
            error,
        }
    }

    // #[inline]
    // pub(crate) fn error(&self) -> &nom::Err<Error<'i>> {
    //     &self.error
    // }
}

#[must_use]
pub(crate) enum DebugError<'i, 'a, 'b: 'a> {
    ParseFailed(ParseFailed<'i, 'a, 'b>),
    Formatting(core::fmt::Error),
}

impl<'i, 'a, 'b: 'a> DebugError<'i, 'a, 'b> {
    fn finish(self) -> core::fmt::Result {
        match self {
            Self::ParseFailed(failure) => match failure.state {
                ParseFailedState::List(mut list) => list.entry(&failure.error).finish(),
                ParseFailedState::Struct(mut debug) => {
                    debug.field("error", &failure.error).finish()
                }
            },
            Self::Formatting(e) => Err(e),
        }
    }
}

impl From<core::fmt::Error> for DebugError<'_, '_, '_> {
    #[inline]
    fn from(error: core::fmt::Error) -> Self {
        Self::Formatting(error)
    }
}

impl<'i, 'a, 'b: 'a> From<ParseFailed<'i, 'a, 'b>> for DebugError<'i, 'a, 'b> {
    #[inline]
    fn from(failure: ParseFailed<'i, 'a, 'b>) -> Self {
        Self::ParseFailed(failure)
    }
}

pub(crate) type Result<'i, 'a, 'b> = core::result::Result<&'i [u8], DebugError<'i, 'a, 'b>>;

/// Variant of the [`Debug`] formatting trait that accounts for parser errors.
pub(crate) trait DebugParse<'i> {
    fn format<'a, 'b: 'a>(self, f: &'a mut Formatter<'b>) -> Result<'i, 'a, 'b>;
}

enum PrintOnceState<'i, P: DebugParse<'i>> {
    Initial(P),
    Printed(core::result::Result<&'i [u8], nom::Err<Error<'i>>>),
}

pub(crate) struct PrintOnce<'i, P: DebugParse<'i>> {
    state: core::cell::Cell<PrintOnceState<'i, P>>,
    _marker: core::marker::PhantomData<fn() -> &'i [u8]>,
}

impl<'i, P: DebugParse<'i>> PrintOnce<'i, P> {
    #[inline]
    pub(crate) fn new(parser: P) -> Self {
        Self {
            state: core::cell::Cell::new(PrintOnceState::Initial(parser)),
            _marker: core::marker::PhantomData,
        }
    }

    pub(crate) fn expect_result(self) -> core::result::Result<&'i [u8], nom::Err<Error<'i>>> {
        match self.state.into_inner() {
            PrintOnceState::Initial(_) => panic!("parser output was not yet formatted"),
            PrintOnceState::Printed(result) => result,
        }
    }
}

impl<'i, P: DebugParse<'i>> Debug for PrintOnce<'i, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.state.replace(PrintOnceState::Printed(Ok(&[]))) {
            PrintOnceState::Initial(parser) => match parser.format(f) {
                Ok(input) => {
                    self.state.set(PrintOnceState::Printed(Ok(input)));
                    Ok(())
                }
                Err(error) => {
                    if let DebugError::ParseFailed(failed) = &error {
                        self.state
                            .set(PrintOnceState::Printed(Err(failed.error.clone())));
                    }

                    error.finish()
                }
            },
            PrintOnceState::Printed(_) => panic!("cannot format parser output more than once"),
        }
    }
}
