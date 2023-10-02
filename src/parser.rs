/// Default error type in [`nom-wasm`].
///
/// [`nom-wasm`]: crate
pub type Error<'a> = nom::error::Error<&'a [u8]>;

/// Type alias for the result of parsing functions in [`nom-wasm`].
///
/// [`nom-wasm`]: crate
pub type Parsed<'a, T, E = Error<'a>> = nom::IResult<&'a [u8], T, E>;

pub trait ParseFailed<'a>:
    nom::error::ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>
{
}

impl<'a, E> ParseFailed<'a> for E where
    E: nom::error::ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>
{
}
