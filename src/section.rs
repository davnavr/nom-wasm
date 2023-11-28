//! Contains functions for parsing WebAssembly sections.
//!
//! A sequence of sections is a common structure in the WebAssembly binary format, used not only in
//! the [binary encoding for modules], but also in some custom sections.
//!
//! [binary encoding for modules]: https://webassembly.github.io/spec/core/binary/modules.html#binary-section

use crate::{
    error::{AddCause as _, ErrorCause, ErrorKind, ErrorSource},
    input::Result,
    Parsed,
};
use nom::ToUsize;

/// Represents a [WebAssembly section], typically a [section within a module].
///
/// [WebAssembly section]: https://webassembly.github.io/spec/core/binary/modules.html#sections
/// [section within a module]: https://webassembly.github.io/spec/core/binary/modules.html#binary-section
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Section<'a> {
    /// The [*id*] for this section.
    ///
    /// [*id*]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    pub id: u8,
    /// The contents of the section.
    pub contents: &'a [u8],
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

        let (input, length) =
            crate::values::leb128_u32(input).add_cause(ErrorCause::SectionLength)?;

        if let Some(contents) = input.get(..length.to_usize()) {
            Ok((&input[..length.to_usize()], Self { id, contents }))
        } else {
            Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Eof,
                ErrorCause::SectionContents(crate::error::LengthMismatch {
                    expected: length,
                    actual: input.len().try_into().unwrap_or(u32::MAX),
                }),
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

impl core::fmt::Debug for Section<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Section")
            .field("id", &crate::hex::Hex(self.id))
            .field("contents", &crate::hex::Bytes(self.contents))
            .finish()
    }
}

/// Parses a sequence of WebAssembly [`Section`]s.
///
/// This is an [`Iterator`] that yields both the remaining input before the [`Section`] was parsed
/// and the [`Section`] itself.
#[derive(Default)]
#[must_use = "call Iterator::next() or .finish()"]
pub struct Sequence<'a, E: ErrorSource<'a>> {
    input: &'a [u8],
    _marker: core::marker::PhantomData<dyn nom::Parser<&'a [u8], (), E>>,
}

impl<'a, E: ErrorSource<'a>> Clone for Sequence<'a, E> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'a, E: ErrorSource<'a>> Sequence<'a, E> {
    /// Creates a new [`Sequence`] from the entirety of the given `input`.
    #[inline]
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            _marker: core::marker::PhantomData,
        }
    }

    /// Parses all of the remaining [`Section`]s.
    pub fn finish(mut self) -> Result<(), E> {
        while self.next().transpose()?.is_some() {}
        debug_assert!(self.input.is_empty());
        Ok(())
    }
}

impl<'a, E: ErrorSource<'a>> crate::input::AsInput<'a> for Sequence<'a, E> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.input
    }
}

impl<'a, E: ErrorSource<'a>> Iterator for Sequence<'a, E> {
    /// The [`Section`] that was parsed, and the remaining input starting with the [`Section`]
    /// that was parsed.
    type Item = Result<(&'a [u8], Section<'a>), E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            None
        } else {
            let start = self.input;
            Some(match Section::parse(self.input) {
                Ok((remaining, section)) => {
                    self.input = remaining;
                    Ok((start, section))
                }
                Err(err) => {
                    // Stop parsing early
                    self.input = &[];
                    Err(err)
                }
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::from(!self.input.is_empty()), None)
    }
}

impl<'a, E: ErrorSource<'a>> core::iter::FusedIterator for Sequence<'a, E> {}

impl<'a, E: ErrorSource<'a>> core::fmt::Debug for Sequence<'a, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Sequence")
            .field("remaining", &crate::hex::Bytes(self.input))
            .finish()
    }
}
