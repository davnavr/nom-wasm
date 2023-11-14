use crate::{
    error::{self, ErrorSource},
    parser::Parser as _,
    types::{self, ValTypeParser},
};
use nom::Parser;

const FUNC_TYPE_TAG: u8 = 0x60;

/// Parses a WebAssembly [result type], which encodes the argument and result types within a
/// [function type].
///
/// [result type]: https://webassembly.github.io/spec/core/binary/types.html#result-types
/// [function type]: func_type_with()
pub struct ResultTypeIter<'a, E: ErrorSource<'a>> {
    iterator: crate::values::VectorIter<'a, types::ValType, E, ValTypeParser>,
    result: crate::input::Result<(), E>,
}

impl<'a, E: ErrorSource<'a>> ResultTypeIter<'a, E> {
    fn new(input: &'a [u8]) -> crate::input::Result<Self, E> {
        crate::values::VectorIter::with_parsed_length(input, ValTypeParser).map(|iterator| Self {
            iterator,
            result: Ok(()),
        })
    }

    fn finish(self) -> crate::Parsed<'a, (), E> {
        self.result?;
        self.iterator
            .finish()
            .map(|(input, ValTypeParser)| (input, ()))
    }
}

impl<'a, E: ErrorSource<'a>> Iterator for ResultTypeIter<'a, E> {
    type Item = types::ValType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.result.is_ok() {
            match self.iterator.next()? {
                Ok(ty) => Some(ty),
                Err(err) => {
                    self.result = Err(err);
                    None
                }
            }
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

impl<'a, E: ErrorSource<'a>> core::iter::FusedIterator for ResultTypeIter<'a, E> {}

impl<'a, E> core::fmt::Debug for ResultTypeIter<'a, E>
where
    E: core::fmt::Debug + ErrorSource<'a>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        if let Err(error) = &self.result {
            list.entry(error);
        } else {
            let mut items = Self {
                iterator: self.iterator.clone(),
                result: Ok(()),
            };
            list.entries(&mut items);
            if let Err(error) = items.finish() {
                list.entry(&error);
            }
        }

        list.finish()
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
        error::ErrorCause::InvalidTag(error::InvalidTag::FuncType(input.first().copied()))
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
