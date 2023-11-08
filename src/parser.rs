use crate::error::{AddCause as _, ErrorCause, ErrorSource};

pub(crate) struct WithErrorCause<P, F> {
    parser: P,
    error: F,
}

impl<'a, O, E, P, F> nom::Parser<&'a [u8], O, E> for WithErrorCause<P, F>
where
    P: nom::Parser<&'a [u8], O, E>,
    F: FnMut(&'a [u8]) -> ErrorCause,
    E: ErrorSource<'a>,
{
    fn parse(&mut self, input: &'a [u8]) -> nom::IResult<&'a [u8], O, E> {
        self.parser
            .parse(input)
            .add_cause_with(|| (self.error)(input))
    }
}

pub(crate) trait Parser<'a, O, E>: nom::Parser<&'a [u8], O, E> + Sized
where
    E: ErrorSource<'a>,
{
    #[inline]
    fn with_error_cause<F>(self, f: F) -> WithErrorCause<Self, F>
    where
        F: FnMut(&'a [u8]) -> ErrorCause,
    {
        WithErrorCause {
            parser: self,
            error: f,
        }
    }
}

impl<'a, O, E, P> Parser<'a, O, E> for P
where
    P: nom::Parser<&'a [u8], O, E>,
    E: ErrorSource<'a>,
{
}
