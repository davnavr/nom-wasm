//! Types and functions for parsing the contents of [custom sections] within a WebAssembly module.
//!
//! [custom sections]: https://webassembly.github.io/spec/core/binary/modules.html#custom-section

use crate::{
    error::{AddCause as _, ErrorCause, ErrorSource},
    input,
    section::Section,
};

/// Represents a [*custom section*] within a [WebAssembly module].
///
/// [*custom section*]: https://webassembly.github.io/spec/core/appendix/custom.html
/// [`Module`]: crate::module::Module
/// [WebAssembly module]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CustomSection<'a> {
    /// The [`name`] of the custom section.
    ///
    /// [`name`]: https://webassembly.github.io/spec/core/binary/modules.html#custom-section
    pub name: &'a str,
    /// The contents of the **`custom`**` section.
    pub contents: &'a [u8],
}

impl<'a> CustomSection<'a> {
    /// The byte [*id*] for custom sections in a WebAssembly module.
    ///
    /// [*id*]: Section::id
    pub const ID: u8 = crate::module::ModuleSectionId::Custom as u8;

    /// Parses a custom section from a [`Section`]'s [`contents`].
    ///
    /// [`contents`]: Section::contents
    pub fn parse<E: ErrorSource<'a>>(input: &'a [u8]) -> input::Result<Self, E> {
        crate::values::name(input)
            .add_cause(ErrorCause::CustomSectionName)
            .map(|(contents, name)| Self { name, contents })
    }

    /// Attempts to interpret the contents of a WebAssembly module [`Section`] as a [`CustomSection`].
    ///
    /// # Errors
    ///
    /// Returns `Err(_)` if the section is **not** a custom section
    /// (the module [*id* is **not** `0`]), or `Ok(Err(_))` if the custom section [`name`] could
    /// not be parsed.
    ///
    /// [*id* is *not* `0`]: CustomSection::ID
    /// [`name`]: CustomSection::name
    pub fn interpret_section<'b, E: ErrorSource<'a>>(
        section: &'b Section<'a>,
    ) -> Result<input::Result<Self, E>, &'b Section<'a>> {
        if section.id == Self::ID {
            Ok(Self::parse(section.contents))
        } else {
            Err(section)
        }
    }
}
