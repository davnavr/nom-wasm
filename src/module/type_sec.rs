use crate::{
    error::{AddCause as _, ErrorCause, ErrorSource},
    input::Result,
    storage::Vector,
    types::{self, BuildFuncType, FuncType, FuncTypeParser, ParseFuncType},
};
use nom::ToUsize as _;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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
    /// Parses a *type section* from a section's contents.
    pub fn parse<E: ErrorSource<'a>>(contents: &'a [u8]) -> Result<Self, E> {
        let (types, count) =
            crate::values::leb128_u32(contents).add_cause(ErrorCause::VectorLength)?;

        Ok(Self { count, types })
    }

    /// Returns a struct to parse the contents of the *type section*, using the provided
    /// [`ParseFuncType`] implementation.
    pub fn parse_contents_with<P, E>(&self, parser: P) -> Result<P, E>
    where
        P: ParseFuncType,
        E: ErrorSource<'a>,
    {
        let mut f = FuncTypeParser::new(parser);
        let (input, ()) = crate::values::sequence(self.types, self.count.to_usize(), &mut f)?;
        nom::combinator::eof(input)?;
        Ok(f.into_inner())
    }

    /// Parse the contents of the *type section* with a given [`ParseFuncType`] implementation.
    #[inline]
    pub fn parse_contents<P, E>(&self) -> Result<P, E>
    where
        P: ParseFuncType + Default,
        E: ErrorSource<'a>,
    {
        self.parse_contents_with(P::default())
    }

    /// Parse all of the contents of the *type section*, appending each parsed [`FuncType`] to the
    /// end of the `destination` [`Vector`].
    pub fn parse_all_contents_with<E, V, B>(
        &self,
        destination: &mut V,
        buffer: &mut BuildFuncType<B>,
    ) -> Result<(), E>
    where
        E: ErrorSource<'a>,
        V: Vector<Item = FuncType<B>>,
        B: Vector<Item = types::ValType> + Clone,
    {
        let count = self.count.to_usize();
        destination.reserve(count);
        let (input, ()) = crate::values::sequence(self.types, count, |input| {
            let (input, func_type) = FuncType::parse::<E, B>(input, buffer)?;
            destination.push(func_type);
            Ok((input, ()))
        })?;
        nom::combinator::eof(input)?;
        Ok(())
    }

    /// Parses all of the contents of the *type section*, returning a [`Vec`] of all of the parsed
    /// [`FuncType`]s.
    ///
    /// [`Vec`]: alloc::vec::Vec
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    pub fn parse_all_contents<E: ErrorSource<'a>>(
        &self,
        buffer: &mut BuildFuncType<Vec<types::ValType>>,
    ) -> Result<Vec<FuncType<Vec<types::ValType>>>, E> {
        let mut types = Vec::with_capacity(self.count.to_usize());
        self.parse_all_contents_with(&mut types, buffer)?;
        Ok(types)
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
