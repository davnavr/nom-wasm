//! Contains functions for parsing the WebAssembly module preamble [module preamble], which marks
//! the start of a WebAssembly module.
//!
//! [module preamble]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module

use crate::parser::{ParseFailed, Parsed};

/// The 4-byte magic number, placed at the start of the preamble, which indicates that a file
/// is a WebAssembly module.
pub const MAGIC: [u8; 4] = *b"\0asm";

/// The current version of the binary format supported by [`nom-wasm`], placed after the [`magic`] field.
///
/// [`nom-wasm`]: crate
/// [`magic`]: MAGIC
pub const RECOGNIZED_VERSION: [u8; 4] = 1u32.to_le_bytes();

fn parse_magic<'a, E: ParseFailed<'a>>(input: &'a [u8]) -> Parsed<'a, (), E> {
    nom::error::context(
        "not a valid WebAssembly module, missing WebAssembly magic",
        nom::bytes::complete::tag(MAGIC),
    )(input)
    .map(|(remaining, _)| (remaining, ()))
}

/// Parses a module preamble, checking that the contents of its `version` field matches the
/// [`RECOGNIZED_VERSION`].
///
/// To handle different version values, use [`parse_any()`].
pub fn parse<'a, E: ParseFailed<'a>>(input: &'a [u8]) -> Parsed<'a, (), E> {
    parse_magic(input)?;
    let (remaining, _) = nom::error::context(
        "encountered unsupported WebAssembly version",
        nom::bytes::complete::tag(RECOGNIZED_VERSION),
    )(input)?;
    Ok((remaining, ()))
}

/// Parses a module preamble, returning the contents of its `version` field.
///
/// If you don't want to handle special version values, use [`parse()`] instead.
pub fn parse_any<'a, E: ParseFailed<'a>>(input: &'a [u8]) -> Parsed<'a, [u8; 4], E> {
    parse_magic(input)?;
    nom::bytes::complete::take(4usize)(input)
        .map(|(remaining, version)| (remaining, version.try_into().unwrap()))
}
