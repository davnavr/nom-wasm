use crate::{
    error::{self, ErrorSource},
    input::Result,
    types::ResultType,
};
use core::marker::PhantomData;

/// Trait for parsing a WebAssembly [function type].
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub trait ParseFuncType<'a, E: ErrorSource<'a>> {
    /// Handles parsing the parameter types of a function.
    #[inline]
    fn parameters(&mut self, parameters: &mut ResultType<'a, E>) -> Result<(), E> {
        let _ = parameters;
        Ok(())
    }

    /// Handles parsing the result types of a function.
    ///
    /// Called after the [`ParseFuncType::parameters()`].
    #[inline]
    fn results(&mut self, results: &mut ResultType<'a, E>) -> Result<(), E> {
        let _ = results;
        Ok(())
    }
}

impl<'a, P, E> ParseFuncType<'a, E> for &mut P
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    #[inline]
    fn parameters(&mut self, parameters: &mut ResultType<'a, E>) -> Result<(), E> {
        P::parameters(self, parameters)
    }

    #[inline]
    fn results(&mut self, results: &mut ResultType<'a, E>) -> Result<(), E> {
        P::results(self, results)
    }
}

impl<'a, E: ErrorSource<'a>> ParseFuncType<'a, E> for () {}

const _OBJECT_SAFE: PhantomData<&'static dyn ParseFuncType<'static, ()>> = PhantomData;

/// Provides a [`nom::Parser`] implementation for an existing [`ParseFuncType`] implementation.
pub struct FuncTypeParser<'a, P, E>
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    parser: P,
    _marker: PhantomData<fn(&'a [u8]) -> E>,
}

impl<'a, P, E> Default for FuncTypeParser<'a, P, E>
where
    P: ParseFuncType<'a, E> + Default,
    E: ErrorSource<'a>,
{
    #[inline]
    fn default() -> Self {
        Self::new(P::default())
    }
}

impl<'a, P, E> FuncTypeParser<'a, P, E>
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    #[allow(missing_docs)]
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}

impl<'a, P, E> nom::Parser<&'a [u8], (), E> for FuncTypeParser<'a, P, E>
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, (), E> {
        crate::types::func_type(input, &mut self.parser).map(|(input, _)| (input, ()))
    }
}

impl<'a, P, E> Clone for FuncTypeParser<'a, P, E>
where
    P: ParseFuncType<'a, E> + Clone,
    E: ErrorSource<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.parser.clone())
    }
}

impl<'a, P: ParseFuncType<'a, E> + Copy, E: ErrorSource<'a>> Copy for FuncTypeParser<'a, P, E> {}

impl<'a, P, E> core::fmt::Debug for FuncTypeParser<'a, P, E>
where
    P: ParseFuncType<'a, E> + core::fmt::Debug,
    E: ErrorSource<'a>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("FuncTypeParser").field(&self.parser).finish()
    }
}

const FUNC_TYPE_TAG: u8 = 0x60;

//fn func_type_no_tag // parse with FUNC_TYPE_TAG

/// Parses a WebAssembly [function type] from the given `input`.
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub fn func_type<'a, P, E>(input: &'a [u8], mut parsers: P) -> crate::Parsed<'a, P, E>
where
    P: ParseFuncType<'a, E>,
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

    let mut parameter_types = ResultType::parse_length_32(input)?;
    parsers.parameters(&mut parameter_types)?;
    let (input, _) = parameter_types.finish()?;

    let mut result_types = ResultType::parse_length_32(input)?;
    parsers.results(&mut result_types)?;
    let (input, _) = result_types.finish()?;

    Ok((input, parsers))
}

//#[cfg(feature = "alloc")]
//pub struct FuncType { types: Box<[ValType]> }
