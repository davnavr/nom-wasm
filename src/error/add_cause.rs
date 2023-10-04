use crate::{
    error::{ErrorCause, ErrorSource},
    Parsed,
};

pub(crate) trait AddCause<'a, T, E: ErrorSource<'a>> {
    fn add_cause_with<F: FnOnce() -> ErrorCause>(self, f: F) -> Parsed<'a, T, E>;

    fn add_cause(self, cause: ErrorCause) -> Parsed<'a, T, E>;
}

impl<'a, T, E: ErrorSource<'a>> AddCause<'a, T, E> for Parsed<'a, T, E> {
    #[inline]
    fn add_cause_with<F: FnOnce() -> ErrorCause>(self, f: F) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.map(|e| e.with_cause(f()))),
        }
    }

    #[inline]
    fn add_cause(self, cause: ErrorCause) -> Parsed<'a, T, E> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.map(|e| e.with_cause(cause))),
        }
    }
}
