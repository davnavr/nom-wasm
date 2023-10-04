//! Contains functions for parsing WebAssembly sections.
//!
//! A sequence of sections is a common structure in the WebAssembly binary format, used not only in
//! the [binary encoding for modules], but also in some custom sections.
//!
//! [binary encoding for modules]: https://webassembly.github.io/spec/core/binary/modules.html#binary-section

use crate::{
    error::{AddCause as _, ErrorCause, ErrorKind, ErrorSource},
    Parsed,
};
use nom::ToUsize;

/// Represents a [WebAssembly section], typically a [section within a module].
///
/// [WebAssembly section]: https://webassembly.github.io/spec/core/binary/modules.html#sections
/// [section within a module]: https://webassembly.github.io/spec/core/binary/modules.html#binary-section
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Section<'a> {
    /// The [*id*] for this section.
    ///
    /// [*id*]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    pub id: u8,
    /// The contents of the section.
    pub contents: &'a [u8], // TODO: Make a DebugHex struct
}

impl<'a> Section<'a> {
    /// Parses a [`Section`] with the given `id` from the given `input`.
    pub fn parse<E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, Self, E> {
        let (input, id) = if let Some((id, remaining)) = input.split_first() {
            (remaining, *id)
        } else {
            return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Tag,
                ErrorCause::SectionId,
            )));
        };

        let (input, length) = crate::leb128::u32(input).add_cause(ErrorCause::SectionLength)?;

        if let Some(contents) = input.get(..length.to_usize()) {
            Ok((&input[..length.to_usize()], Self { id, contents }))
        } else {
            Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Eof,
                ErrorCause::SectionContents {
                    expected: length,
                    actual: input.len().try_into().unwrap_or(u32::MAX),
                },
            )))
        }
    }

    /// Creates a new [`Section`] with the given [*id*] and `contents`.
    ///
    /// [*id*]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    #[inline]
    pub fn new(id: u8, contents: &'a [u8]) -> Self {
        Self { id, contents }
    }

    /*
    /// Returns a [`Debug`] implementation that attempts to interpret the contents as a WebAssembly
    /// module section.
    #[inline]
    pub fn debug_module(&self) -> DebugModuleSection<'_, I> {
        DebugModuleSection::new(self)
    }
    */
}

//pub fn sequence<'a, E, T, F>(input: &'a [u8])
//where
//    F: FnMut(Section),
//    E: ParseFailed<'a>
