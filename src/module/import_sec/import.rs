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
    pub desc: crate::module::ImportDesc,
}

impl<'a> Import<'a> {
    #[allow(missing_docs)]
    pub fn parse<E: crate::error::ErrorSource<'a>>(input: &'a [u8]) -> crate::Parsed<'a, Self, E> {
        use crate::error::{AddCause as _, ErrorCause, ImportComponent};

        let (input, module) = crate::values::name(input)
            .add_cause(input, ErrorCause::Import(ImportComponent::Module))?;

        let (input, name) = crate::values::name(input)
            .add_cause(input, ErrorCause::Import(ImportComponent::Name))?;

        let (input, desc) = crate::module::ImportDesc::parse(input)?;

        Ok((input, Self { module, name, desc }))
    }
}

/// Provides a [`nom::Parser`] implementation for [`Import::parse()`].
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct ImportParser;

impl<'a, E: crate::error::ErrorSource<'a>> nom::Parser<&'a [u8], Import<'a>, E> for ImportParser {
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, Import<'a>, E> {
        Import::parse(input)
    }
}
