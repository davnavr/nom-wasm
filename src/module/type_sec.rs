use crate::{
    error::ErrorSource,
    sequence::{self, VectorParser},
    types::{FuncTypeParser, ParseFuncType},
};

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
    types: VectorParser<'a, (), E, FuncTypeParser<'a, P, E>>,
}

impl<'a, P, E> TypesComponent<'a, P, E>
where
    P: ParseFuncType<'a, E>,
    E: ErrorSource<'a>,
{
    #[inline]
    fn new(types: VectorParser<'a, (), E, FuncTypeParser<'a, P, E>>) -> Self {
        Self { types }
    }

    //fn finish
}

//Sequence

//Vector

//IntoIterator

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

impl<'a, P, E> core::fmt::Debug for TypesComponent<'a, P, E>
where
    P: ParseFuncType<'a, E> + Clone,
    E: ErrorSource<'a>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!() // TODO: Make helper debug struct for FuncType's
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
}

impl core::fmt::Debug for TypeSec<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!() // TODO: Make helper debug struct for FuncType's
    }
}
