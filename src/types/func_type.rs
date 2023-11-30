use crate::{error::ErrorSource, parser::Parser as _};
use nom::Parser;

const FUNC_TYPE_TAG: u8 = 0x60;

/// Parses a WebAssembly [result type], which encodes the argument and result types within a
/// [function type].
///
/// [result type]: https://webassembly.github.io/spec/core/binary/types.html#result-types
/// [function type]: func_type_with()
#[derive(Clone)]
pub struct ResultTypeIter<'a, E: ErrorSource<'a>> {
    types: crate::values::SequenceIter<
        'a,
        crate::values::Vector<'a, crate::types::ValType, E, crate::types::ValTypeParser>,
    >,
}

impl<'a, E: ErrorSource<'a>> ResultTypeIter<'a, E> {
    fn new(input: &'a [u8]) -> crate::input::Result<Self, E> {
        crate::values::Vector::with_parsed_length(input, crate::types::ValTypeParser).map(|types| {
            Self {
                types: types.into(),
            }
        })
    }

    #[inline]
    fn finish(self) -> crate::Parsed<'a, (), E> {
        self.types
            .finish()
            .map(|types| (crate::input::AsInput::as_input(&types), ()))
    }
}

impl<'a, E: ErrorSource<'a>> Iterator for ResultTypeIter<'a, E> {
    type Item = crate::types::ValType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        (&mut self.types).next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        crate::values::Sequence::size_hint(self.types.sequence())
    }
}

impl<'a, E: ErrorSource<'a>> core::iter::FusedIterator for ResultTypeIter<'a, E> {}

impl<'a, E> core::fmt::Debug for ResultTypeIter<'a, E>
where
    E: core::fmt::Debug + Clone + ErrorSource<'a>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.types, f)
    }
}

/// Parses a WebAssembly [function type], also known as a **`functype`**.
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub fn func_type_with<'a, E, A, B, C, I, P, R>(
    mut init: I,
    mut param_types: P,
    mut result_types: R,
) -> impl Parser<&'a [u8], C, E>
where
    E: ErrorSource<'a>,
    I: FnMut() -> A,
    P: FnMut(A, &mut ResultTypeIter<'a, E>) -> B,
    R: FnMut(B, &mut ResultTypeIter<'a, E>) -> C,
{
    let mut func_type_tag = nom::bytes::streaming::tag([FUNC_TYPE_TAG]).with_error_cause(|input| {
        crate::error::ErrorCause::InvalidTag(crate::error::InvalidTag::FuncType(
            input.first().copied(),
        ))
    });

    move |input| -> crate::Parsed<'a, _, E> {
        let (input, _) = func_type_tag.parse(input)?;
        let state = init();
        let mut parameters_iter = ResultTypeIter::new(input)?;
        let state = param_types(state, &mut parameters_iter);
        let (input, ()) = parameters_iter.finish()?;
        let mut results_iter = ResultTypeIter::new(input)?;
        let state = result_types(state, &mut results_iter);
        let (input, ()) = results_iter.finish()?;
        Ok((input, state))
    }
}
