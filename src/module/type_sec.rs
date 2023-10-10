use crate::{
    error::ErrorSource,
    types::{FuncTypeParser, ParseFuncType},
};

/// Represents the [*type section*] of a WebAssembly module.
///
/// This corresponds to the [**types** component] of a WebAssembly module.
///
/// [*type section*]: https://webassembly.github.io/spec/core/binary/modules.html#type-section
/// [**types** component]: https://webassembly.github.io/spec/core/syntax/modules.html#types
#[derive(Clone, Copy, Default)]
#[must_use]
pub struct TypeSec<'a> {
    count: u32,
    types: &'a [u8],
}

impl<'a> TypeSec<'a> {
    /// Returns a struct to parse the contents of the *type section*, using the provided
    /// [`ParseFuncType`] implementation.
    pub fn parse_contents_with<P, E>(&self, parser: P) -> crate::Parsed<'a, P, E>
    where
        P: ParseFuncType,
        E: ErrorSource<'a>,
    {
        let mut f = FuncTypeParser::new(parser);
        crate::values::sequence(self.types, nom::ToUsize::to_usize(&self.count), &mut f)
            .map(|(input, ())| (input, f.into_inner()))
    }

    /// Parse the contents of the *type section* with a given [`ParseFuncType`] implementation.
    #[inline]
    pub fn parse_contents<P, E>(&self) -> crate::Parsed<'a, P, E>
    where
        P: ParseFuncType + Default,
        E: ErrorSource<'a>,
    {
        self.parse_contents_with(P::default())
    }
}

impl<'a> crate::input::AsInput<'a> for TypeSec<'a> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.types
    }
}

impl core::fmt::Debug for TypeSec<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //debug_types(nom::ToUsize::to_usize(&self.count), self.types, f)
        // TODO: Pretty print the func types instead
        f.debug_struct("TypeSec")
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}
