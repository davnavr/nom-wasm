//! Contains functions for parsing the WebAssembly module preamble [module preamble], which marks
//! the start of a WebAssembly module.
//!
//! [module preamble]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module

use crate::{
    error::{AddCause, ErrorCause, ErrorSource},
    Parsed,
};

/// The 4-byte magic number, placed at the start of the preamble, which indicates that a file
/// is a WebAssembly module.
pub const MAGIC: [u8; 4] = *b"\0asm";

/// The current version of the binary format supported by [`nom-wasm`], placed after the [`magic`] field.
///
/// [`nom-wasm`]: crate
/// [`magic`]: MAGIC
pub const RECOGNIZED_VERSION: [u8; 4] = 1u32.to_le_bytes();

fn parse_magic<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, (), E> {
    nom::bytes::complete::tag(MAGIC)(input)
        .map(|(remaining, _)| (remaining, ()))
        .add_cause_with(|| {
            ErrorCause::PreambleMagic(
                arrayvec::ArrayVec::try_from(&input[..input.len().min(4)]).unwrap(),
            )
        })
}

/// Parses a module preamble, checking that the contents of its `version` field matches the
/// [`RECOGNIZED_VERSION`].
///
/// To handle different version values, use [`parse_any()`].
pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, (), E> {
    parse_magic(input)?;
    nom::bytes::complete::tag(RECOGNIZED_VERSION)(input)
        .map(|(remaining, _)| (remaining, ()))
        .add_cause_with(|| {
            ErrorCause::PreambleVersion(
                input
                    .get(..4)
                    .map(|version| u32::from_le_bytes(version.try_into().unwrap())),
            )
        })
}

/// Parses a module preamble, returning the contents of its `version` field.
///
/// If you don't want to handle special version values, use [`parse()`] instead.
pub fn parse_any<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, [u8; 4], E> {
    parse_magic(input)?;
    nom::bytes::complete::take(4usize)(input)
        .map(|(remaining, version)| (remaining, version.try_into().unwrap()))
        .add_cause(ErrorCause::PreambleVersion(None))
}
