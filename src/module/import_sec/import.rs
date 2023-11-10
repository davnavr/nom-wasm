use crate::{
    error::{self, AddCause as _},
    module::ImportDesc,
};

/// Represents a [WebAssembly **`import`**].
///
/// Note that importing more than one memory requires the [multi-memory proposal].
///
/// [WebAssembly **`import`**]: https://webassembly.github.io/spec/core/binary/modules.html#import-section
/// [multi-memory proposal]: https://github.com/WebAssembly/multi-memory
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct Import<'a> {
    /// The name of the module that this import originates from.
    pub module: &'a str,
    /// The name of the import.
    pub name: &'a str,
    /// The description for the import.
    pub desc: ImportDesc,
}

impl<'a> Import<'a> {
    #[allow(missing_docs)]
    pub fn parse<E: error::ErrorSource<'a>>(input: &'a [u8]) -> crate::Parsed<'a, Self, E> {
        let (input, module) = crate::values::name(input)
            .add_cause(error::ErrorCause::Import(error::ImportComponent::Module))?;

        let (input, name) = crate::values::name(input)
            .add_cause(error::ErrorCause::Import(error::ImportComponent::Name))?;

        let (input, desc) = ImportDesc::parse(input)?;

        Ok((input, Self { module, name, desc }))
    }
}

/// Provides a [`nom::Parser`] implementation for [`Import::parse()`].
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct ImportParser;

impl<'a, E: error::ErrorSource<'a>> nom::Parser<&'a [u8], Import<'a>, E> for ImportParser {
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, Import<'a>, E> {
        Import::parse(input)
    }
}
