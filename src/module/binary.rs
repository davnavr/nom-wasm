use crate::{error::ErrorSource, input::Result, module};

/// Parses the [WebAssembly module preamble], then returns a [`ModuleSectionSequence`] for parsing
/// the module's sections.
///
/// # Errors
///
/// Returns an error if the preamble could not be parsed.
///
/// [WebAssembly module preamble]: module::preamble
/// [`ModuleSectionSequence`]: module::ModuleSectionSequence
pub fn parse_binary_sections<'a, E>(
    input: &'a [u8],
) -> Result<module::ModuleSectionSequence<'a, E>, E>
where
    E: ErrorSource<'a>,
{
    module::preamble::parse(input).map(|(input, ())| module::ModuleSectionSequence::new(input))
}

//pub fn parse_binary_with_unknown(init: I, f: F) -> impl nom::Parser

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
    /// Creates a [`Module`] from its `sections`, while also passing *each* section to the given closure.
    ///
    /// Custom sections can be processed by the closure by calling the
    /// [`UnknownModuleSection::to_custom_section()`] method.
    ///
    /// If special handling of unknown sections is not needed, use the
    /// [`Module::from_sections_with`] method instead.
    ///
    /// If no special processing of sections is needed, use the [`Module::from_sections`] method instead.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`Section`] could not be parsed, or any errors from the closure.
    ///
    /// [`UnknownModuleSection::to_custom_section()`]: module::UnknownModuleSection::to_custom_section()
    /// [`Section`]: crate::section::Section
    pub fn from_sections_with_unknown<E, F>(
        mut sections: module::ModuleSectionSequence<'a, E>,
        mut f: F,
    ) -> Result<Self, E>
    where
        E: ErrorSource<'a>,
        F: FnMut(module::UnknownModuleSection<'a>) -> Result<(), E>,
    {
        let mut module = Self::default();

        while let Some(sec) = crate::values::Sequence::parse(&mut sections)? {
            if let Ok(module_section) = sec.to_module_section::<()>() {
                use module::ModuleSection;

                match module_section.clone() {
                    ModuleSection::Custom(_) => (),
                    ModuleSection::Type(type_sec) => module.type_sec = type_sec,
                    ModuleSection::Import(import_sec) => module.import_sec = import_sec,
                }
            }

            f(sec)?;
        }

        Ok(module)
    }

    /// Creates a [`Module`] from its `sections`, while also passing *each* [`ModuleSection`] to the given closure.
    ///
    /// To also process unknown sections, use the [`Module::from_sections_with_unknown`] method instead.
    ///
    /// If special processing is only needed for custom sections, use the [`Module::from_sections_with_custom`] method instead.
    ///
    /// If no special processing of sections is needed, use the [`Module::from_sections`] method instead.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`ModuleSection`] could not be parsed, or if an unknown section was
    /// encountered.
    ///
    /// [`ModuleSection`]: module::ModuleSection
    pub fn from_sections_with<E, F>(
        sections: module::ModuleSectionSequence<'a, E>,
        mut f: F,
    ) -> Result<Self, E>
    where
        E: ErrorSource<'a>,
        F: FnMut(module::ModuleSection<'a>) -> Result<(), E>,
    {
        Self::from_sections_with_unknown(sections, move |sec| {
            sec.to_module_section().cloned().and_then(&mut f)
        })
    }

    /// Creates a [`Module`] from its `sections`, passing *each* encountered [`CustomSection`] to the given closure.
    ///
    /// To also process unknown sections, use the [`Module::from_sections_with_unknown`] method instead.
    ///
    /// If additional processing of non-custom [`ModuleSection`]s is needed, use the
    /// [`Module::from_sections_with`] method instead.
    ///
    /// If no special processing of sections is needed, use the [`Module::from_sections`] method instead.
    ///
    /// # Errors
    ///
    /// See the documentation for the [`Module::from_sections_with`] method for more information.
    ///
    /// [`CustomSection`]: module::custom::CustomSection
    /// [`ModuleSection`]: module::ModuleSection
    pub fn from_sections_with_custom<E, F>(
        sections: module::ModuleSectionSequence<'a, E>,
        mut f: F,
    ) -> Result<Self, E>
    where
        E: ErrorSource<'a>,
        F: FnMut(module::custom::CustomSection<'a>) -> Result<(), E>,
    {
        Self::from_sections_with(sections, move |sec| {
            if let module::ModuleSection::Custom(custom) = sec {
                f(custom)
            } else {
                Ok(())
            }
        })
    }

    /// Creates a [`Module`] from its `sections`, skipping all [`CustomSection`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`ModuleSection`] could not be parsed, or if an unknown section was
    /// encountered.
    ///
    /// [`CustomSection`]: module::custom::CustomSection
    /// [`ModuleSection`]: module::ModuleSection
    #[inline]
    pub fn from_sections<E>(sections: module::ModuleSectionSequence<'a, E>) -> Result<Self, E>
    where
        E: ErrorSource<'a>,
    {
        Self::from_sections_with_custom(sections, |_| Ok(()))
    }

    /// Parses a module from its encoding in the WebAssembly `binary` format, skipping custom sections.
    ///
    /// # Errors
    ///
    /// See the documentation for the [`Module::from_sections`] method for more information.
    #[inline]
    pub fn parse<E: ErrorSource<'a>>(binary: &'a [u8]) -> Result<Self, E> {
        parse_binary_sections(binary).and_then(Self::from_sections)
    }
}
