use crate::{error::ErrorSource, types::ValType, values};

/// Trait for parsing a WebAssembly [result type].
///
/// [result type]: https://webassembly.github.io/spec/core/binary/types.html#result-types
pub trait ParseResultType {
    /// Called when the number of types is parsed.
    fn with_count(&mut self, count: usize);

    /// Called when a [`ValType`] is parsed.
    fn next_type(&mut self, value_type: ValType);
}

impl<P: ParseResultType> ParseResultType for &mut P {
    #[inline]
    fn with_count(&mut self, count: usize) {
        P::with_count(self, count)
    }

    #[inline]
    fn next_type(&mut self, value_type: ValType) {
        P::next_type(self, value_type)
    }
}

const _OBJECT_SAFE: core::marker::PhantomData<&'static dyn ParseResultType> =
    core::marker::PhantomData;

/// Parses a WebAssembly [result type].
///
/// [result type]: https://webassembly.github.io/spec/core/binary/types.html#result-types
pub fn result_type<'a, P, E>(input: &'a [u8], mut parser: P) -> crate::Parsed<'a, P, E>
where
    P: ParseResultType,
    E: ErrorSource<'a>,
{
    let (input, count) = values::vector_length(input)?;
    parser.with_count(count);
    values::sequence(input, count, |input| {
        let (input, value_type) = ValType::parse(input)?;
        parser.next_type(value_type);
        Ok((input, ()))
    })
    .map(|(input, ())| (input, parser))
}
