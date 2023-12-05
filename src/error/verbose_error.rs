#[derive(PartialEq)]
struct Inner<'a> {
    base: crate::error::Error<'a>,
    additional: alloc::vec::Vec<crate::error::Error<'a>>,
}

/// Accumulates information about an error, including its location, the kinds of error that
/// occured, and the reasons why it occured.
#[derive(PartialEq)]
#[repr(transparent)]
pub struct VerboseError<'a>(alloc::boxed::Box<Inner<'a>>);

impl core::fmt::Debug for VerboseError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entry(&self.0.base)
            .entries(&self.0.additional)
            .finish()
    }
}

impl<'a> nom::error::ParseError<&'a [u8]> for VerboseError<'a> {
    #[inline]
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        crate::error::ErrorSource::from_error_cause(input, kind.into())
    }

    #[inline]
    fn append(input: &'a [u8], kind: nom::error::ErrorKind, other: Self) -> Self {
        crate::error::ErrorSource::append_with_cause(input, kind.into(), other)
    }
}

impl<'a> crate::error::ErrorSource<'a> for VerboseError<'a> {
    fn from_error_cause(input: &'a [u8], cause: crate::error::ErrorCause) -> Self {
        Self(alloc::boxed::Box::new(Inner {
            base: crate::error::ErrorSource::from_error_cause(input, cause),
            additional: alloc::vec::Vec::new(),
        }))
    }

    fn append_with_cause(
        input: &'a [u8],
        cause: crate::error::ErrorCause,
        mut other: Self,
    ) -> Self {
        other
            .0
            .additional
            .push(crate::error::ErrorSource::from_error_cause(input, cause));
        other
    }
}
