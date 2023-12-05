use crate::{
    error::{ErrorCause, ErrorSource},
    input::Result,
};

pub(crate) trait AddCause<'a, T, E: ErrorSource<'a>> {
    fn add_cause_with<F: FnOnce() -> (&'a [u8], ErrorCause)>(self, f: F) -> Self;

    fn add_cause(self, input: &'a [u8], cause: ErrorCause) -> Self;
}

impl<'a, T, E: ErrorSource<'a>> AddCause<'a, T, E> for Result<T, E> {
    #[inline]
    fn add_cause_with<F>(self, f: F) -> Self
    where
        F: FnOnce() -> (&'a [u8], ErrorCause),
    {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.map(move |other| {
                let (input, cause) = f();
                E::append_with_cause(input, cause, other)
            })),
        }
    }

    #[inline]
    fn add_cause(self, input: &'a [u8], cause: ErrorCause) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.map(|other| E::append_with_cause(input, cause, other))),
        }
    }
}
