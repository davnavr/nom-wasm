use crate::{
    error::{self, ErrorSource},
    types,
};
use nom::Parser;

/// Trait for parsing a WebAssembly [function type].
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub trait ParseFuncType {
    /// Handles parsing the individual parameter or result types of the function type.
    type ResultType<'a>: types::ParseResultType
    where
        Self: 'a;

    /// Handles parsing the parameter types of a function type.
    fn parameters<'a>(&'a mut self) -> Self::ResultType<'a>;

    /// Handles parsing the result types of a function type.
    ///
    /// Called after the [`ParseFuncType::parameters()`] method.
    fn results<'a>(&'a mut self) -> Self::ResultType<'a>;
}

impl<'b, P: ParseFuncType> ParseFuncType for &'b mut P {
    type ResultType<'a> = P::ResultType<'a> where 'b: 'a;

    #[inline]
    fn parameters<'a>(&'a mut self) -> Self::ResultType<'a> {
        P::parameters(self)
    }

    #[inline]
    fn results<'a>(&'a mut self) -> Self::ResultType<'a> {
        P::results(self)
    }
}

/// Provides a [`nom::Parser`] implementation for an existing [`ParseFuncType`] implementation.
#[derive(Clone, Copy, Debug)]
pub struct FuncTypeParser<P: ParseFuncType> {
    parser: P,
}

impl<P: ParseFuncType> FuncTypeParser<P> {
    #[allow(missing_docs)]
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser }
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn into_inner(self) -> P {
        self.parser
    }
}

impl<'a, P, E> Parser<&'a [u8], (), E> for FuncTypeParser<P>
where
    P: ParseFuncType,
    E: ErrorSource<'a>,
{
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, (), E> {
        func_type(input, &mut self.parser).map(|(input, _)| (input, ()))
    }
}

impl<'a, P, E> Parser<&'a [u8], (), E> for &mut FuncTypeParser<P>
where
    P: ParseFuncType,
    E: ErrorSource<'a>,
{
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, (), E> {
        FuncTypeParser::parse(self, input)
    }
}

const FUNC_TYPE_TAG: u8 = 0x60;

//fn func_type_no_tag // parse without FUNC_TYPE_TAG

/// Parses a WebAssembly [function type].
///
/// If you need to parse a function type with a [`nom::Parser`], use the [`FuncTypeParser<P>`] struct instead.
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub fn func_type<'a, P, E>(input: &'a [u8], mut parsers: P) -> crate::Parsed<'a, P, E>
where
    P: ParseFuncType,
    E: ErrorSource<'a>,
{
    let input = if let Some((&FUNC_TYPE_TAG, input)) = input.split_first() {
        input
    } else {
        return Err(nom::Err::Failure(E::from_error_kind_and_cause(
            input,
            error::ErrorKind::Tag,
            error::ErrorCause::InvalidTag(error::InvalidTag::FuncType(input.first().copied())),
        )));
    };

    let (input, _) = types::result_type(input, parsers.parameters())?;
    let (input, _) = types::result_type(input, parsers.results())?;
    Ok((input, parsers))
}

//#[cfg(feature = "alloc")]
//pub struct FuncType { types: Box<[ValType]> }
