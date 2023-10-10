use crate::{
    debug,
    error::ErrorSource,
    input::AsInput,
    sequence::{self, Vector as _, VectorParser},
    types::{FuncTypeParser, ParseFuncType},
};
use core::fmt::{Debug, Formatter};

type FuncTypeVectorParser<'a, P, E> = VectorParser<'a, (), E, FuncTypeParser<'a, P, E>>;

/// Represents the [**types** component] of a WebAssembly module, a [`Sequence`] of
/// [function types] encoded in the [`TypeSec`]tion.
///
/// [**types** component]: https://webassembly.github.io/spec/core/syntax/modules.html#types
/// [`Sequence`]: sequence::Sequence
/// [function types]: ParseFuncType
pub struct TypesComponent<'a, P, E>
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    types: FuncTypeVectorParser<'a, P, E>,
}

impl<'a, P, E> TypesComponent<'a, P, E>
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    #[inline]
    fn new(types: FuncTypeVectorParser<'a, P, E>) -> Self {
        Self { types }
    }

    //fn finish(self) -> crate::Parser<'a, P, E>
}

sequence::wrap_sequence_impl! {
    for<'a, E, P> TypesComponent<'a, P, E>[types: FuncTypeVectorParser<'a, P, E>]
    where
        P: ParseFuncType<'a, E>,
}

impl<'a, P, E> Clone for TypesComponent<'a, P, E>
where
    P: ParseFuncType<'a, E> + Clone,
    E: ErrorSource<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.types.clone())
    }
}

fn debug_types<'a>(count: usize, types: &'a [u8], f: &mut Formatter<'_>) -> core::fmt::Result {
    let mut list = f.debug_list();
    let parser = |input: &'a [u8]| -> crate::Parsed<'a, _, _> {
        let printer = debug::PrintOnce::new(crate::types::DebugFuncType::new(input));
        list.entry(&printer);
        match printer.expect_result() {
            Ok(input) => Ok((input, ())),
            Err(err) => Err(err),
        }
    };

    let _ = VectorParser::new(types, count, parser).finish();
    list.finish()
}

impl<'a, P, E> Debug for TypesComponent<'a, P, E>
where
    P: ParseFuncType<'a, E> + Clone,
    E: ErrorSource<'a>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        debug_types(self.types.expected_len(), self.types.as_input(), f)
    }
}

/// Represents the [*type section*] of a WebAssembly module.
///
/// [*type section*]: https://webassembly.github.io/spec/core/binary/modules.html#type-section
#[derive(Clone, Copy, Default)]
#[must_use]
pub struct TypeSec<'a> {
    count: u32,
    types: &'a [u8],
}

impl<'a> TypeSec<'a> {
    /// Returns a struct to parse the contents of the *type section*, using the provided
    /// [`ParseFuncType`] implementation.
    #[inline]
    pub fn parse_contents_with<P, E>(&self, parser: P) -> TypesComponent<'a, P, E>
    where
        P: ParseFuncType<'a, E>,
        E: ErrorSource<'a>,
    {
        TypesComponent::new(VectorParser::new(
            self.types,
            nom::ToUsize::to_usize(&self.count),
            FuncTypeParser::new(parser),
        ))
    }

    /// Parse the contents of the *type section* with a given [`ParseFuncType`] implementation.
    #[inline]
    pub fn parse_contents<P, E>(&self) -> TypesComponent<'a, P, E>
    where
        P: ParseFuncType<'a, E> + Default,
        E: ErrorSource<'a>,
    {
        TypesComponent::new(VectorParser::with_length_32(self.count, self.types))
    }

    // TODO: add a pub fn parse_with that takes trait for parsing FuncTypes
}

impl<'a> AsInput<'a> for TypeSec<'a> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.types
    }
}

impl Debug for TypeSec<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        //debug_types(nom::ToUsize::to_usize(&self.count), self.types, f)
        // TODO: Pretty print the func types instead
        f.debug_struct("TypeSec").field("count", &self.count).finish_non_exhaustive()
    }
}
