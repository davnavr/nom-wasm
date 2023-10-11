use crate::{
    error::ErrorSource,
    input::Result,
    module::{self, custom::CustomSection, preamble, ModuleSection, ModuleSectionOrder},
    section::Section,
};

/// Represents a module in the [WebAssembly binary format].
///
/// [WebAssembly binary format]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct Module<'a> {
    pub type_sec: module::TypeSec<'a>,
    pub import_sec: module::ImportSec<'a>,
}

impl<'a> Module<'a> {
    fn parse_module_section<'b, E, F>(
        module: &'b mut Self,
        mut custom_f: F,
    ) -> impl (FnMut(ModuleSection<'a>, Option<ModuleSectionOrder>) -> Result<(), E>) + 'b
    where
        E: ErrorSource<'a>,
        F: (FnMut(CustomSection<'a>, Option<ModuleSectionOrder>) -> Result<(), E>) + 'b,
    {
        move |section, order| {
            match section {
                ModuleSection::Custom(custom_sec) => custom_f(custom_sec, order)?,
                ModuleSection::Type(type_sec) => module.type_sec = type_sec,
                ModuleSection::Import(import_sec) => module.import_sec = import_sec,
            }

            Ok(())
        }
    }

    /// Parses a module from its encoding in the WebAssembly binary format, using the given
    /// closures to handle custom and unrecognized sections.
    ///
    /// See the documentation for the [`module_section_sequence_with_unknown()`] method for more
    /// information.
    ///
    /// # Errors
    ///
    /// Returns an error if the `binary` does not begin with the WebAssembly [**`magic`**], or if a
    /// section could not be fully parsed.
    ///
    /// [`module_section_sequence_with_unknown()`]: module::module_section_sequence_with_unknown
    /// [**`magic`**]: preamble::MAGIC
    pub fn parse_with_custom_and_unknown_sections<E, C, U>(
        binary: &'a [u8],
        custom: C,
        unknown: U,
    ) -> Result<Self, E>
    where
        E: ErrorSource<'a>,
        C: FnMut(CustomSection<'a>, Option<ModuleSectionOrder>) -> Result<(), E>,
        U: FnMut(&'a [u8], Section<'a>, Option<ModuleSectionOrder>) -> Result<(), E>,
    {
        let (input, ()) = preamble::parse(binary)?;
        let mut module = Self::default();
        module::module_section_sequence_with_unknown(
            input,
            Self::parse_module_section(&mut module, custom),
            unknown,
        )?;
        Ok(module)
    }

    /// Parses a module from its encoding in the WebAssembly binary format, passing custom
    /// sections into the given closures.
    ///
    /// # Errors
    ///
    /// Returns an error if a section could not be parsed, or if an unrecognized [`Section`] was
    /// encountered.
    ///
    /// To handle unrecognized sections, use the
    /// [`Module::parse_with_custom_and_unknown_sections()`] method instead.
    pub fn parse_with_custom_sections<E, C>(binary: &'a [u8], custom: C) -> Result<Self, E>
    where
        E: ErrorSource<'a>,
        C: FnMut(CustomSection<'a>, Option<ModuleSectionOrder>) -> Result<(), E>,
    {
        let (input, ()) = preamble::parse(binary)?;
        let mut module = Self::default();
        module::module_section_sequence(input, Self::parse_module_section(&mut module, custom))?;
        Ok(module)
    }

    /// Parses a module from its encoding in the WebAssembly binary format, ignoring custom sections.
    ///
    /// To process custom sections, use the [`Module::parse_with_custom_sections()`] method
    /// instead.
    pub fn parse<E: ErrorSource<'a>>(binary: &'a [u8]) -> Result<Self, E> {
        #[inline]
        fn skip_custom_section<E>(
            _: CustomSection<'_>,
            _: Option<ModuleSectionOrder>,
        ) -> Result<(), E> {
            Ok(())
        }

        Self::parse_with_custom_sections(binary, skip_custom_section)
    }
}
