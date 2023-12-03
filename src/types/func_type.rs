use nom::Parser;

const FUNC_TYPE_TAG: u8 = 0x60;

/// Parses a WebAssembly [result type], which encodes the argument and result types within a
/// [function type].
///
/// [result type]: https://webassembly.github.io/spec/core/binary/types.html#result-types
/// [function type]: func_type_with()
pub type ResultType<'a, E> =
    crate::values::Vector<'a, crate::types::ValType, E, crate::types::ValTypeParser>;

/// Parses a WebAssembly [function type], also known as a **`functype`**.
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub fn func_type_with<'a, E, A, B, C, I, P, R>(
    mut init: I,
    mut param_types: P,
    mut result_types: R,
) -> impl Parser<&'a [u8], C, E>
where
    E: crate::error::ErrorSource<'a>,
    I: FnMut() -> A,
    P: FnMut(A, &mut ResultType<'a, E>) -> crate::input::Result<B, E>,
    R: FnMut(B, &mut ResultType<'a, E>) -> crate::input::Result<C, E>,
{
    use crate::parser::Parser as _;
    let mut func_type_tag = nom::bytes::streaming::tag([FUNC_TYPE_TAG]).with_error_cause(|input| {
        crate::error::ErrorCause::InvalidTag(crate::error::InvalidTag::FuncType(
            input.first().copied(),
        ))
    });

    move |input| -> crate::Parsed<'a, _, E> {
        use crate::input::AsInput as _;

        let (input, _) = func_type_tag.parse(input)?;
        let state = init();
        let mut parameters_iter = ResultType::try_from(input)?;
        let state = param_types(state, &mut parameters_iter)?;
        let input = crate::values::SequenceIter::from(parameters_iter)
            .finish()?
            .as_input();
        let mut results_iter = ResultType::try_from(input)?;
        let state = result_types(state, &mut results_iter)?;
        let input = crate::values::SequenceIter::from(results_iter)
            .finish()?
            .as_input();
        Ok((input, state))
    }
}
