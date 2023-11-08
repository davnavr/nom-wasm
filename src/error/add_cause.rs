use crate::{
    error::{ErrorCause, ErrorSource},
    input::Result,
};

pub(crate) trait AddCause<'a, T, E: ErrorSource<'a>> {
    fn add_cause_with<F: FnOnce() -> ErrorCause>(self, f: F) -> Self;

    fn add_cause(self, cause: ErrorCause) -> Self;
}

impl<'a, T, E: ErrorSource<'a>> AddCause<'a, T, E> for Result<T, E> {
    #[inline]
    fn add_cause_with<F: FnOnce() -> ErrorCause>(self, f: F) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.map(|e| e.with_cause(f()))),
        }
    }

    #[inline]
    fn add_cause(self, cause: ErrorCause) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.map(|e| e.with_cause(cause))),
        }
    }
}
