//! Contains functions for parsing the WebAssembly module preamble [module preamble], which marks
//! the start of a WebAssembly module.
//!
//! [module preamble]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module

use crate::{
    error::{ErrorCause, ErrorSource},
    parser::Parser as _,
    Parsed,
};
use nom::Parser as _;

/// The 4-byte **`magic`** number, placed at the start of the preamble, which indicates that a file
/// is a WebAssembly module.
pub const MAGIC: [u8; 4] = *b"\0asm";

/// The current version of the binary format supported by [`nom-wasm`], placed after the
/// [**`magic`**] field.
///
/// [`nom-wasm`]: crate
/// [**`magic`**]: magic
pub const RECOGNIZED_VERSION: [u8; 4] = 1u32.to_le_bytes();

#[derive(Clone, Copy, Eq, PartialEq)]
enum InvalidMagicLength {
    Empty = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four,
}

/// Error type used when the [WebAssembly **`magic`** number](magic) could not be parsed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct InvalidMagic {
    contents: [u8; 4],
    length: InvalidMagicLength,
}

impl InvalidMagic {
    fn new(input: &[u8]) -> Self {
        let mut contents = [0u8; 4];
        let length = match input.len() {
            0 => InvalidMagicLength::Empty,
            1 => InvalidMagicLength::One,
            2 => InvalidMagicLength::Two,
            3 => InvalidMagicLength::Three,
            _ => InvalidMagicLength::Four,
        };

        contents.copy_from_slice(&input[..length as usize]);
        Self { contents, length }
    }

    /// Gets the value of the **`magic`** field.
    #[inline]
    pub fn value(&self) -> &[u8] {
        &self.contents[..self.length as usize]
    }
}

impl core::fmt::Debug for InvalidMagic {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("InvalidMagic").field(&self.value()).finish()
    }
}

impl core::fmt::Display for InvalidMagic {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("not a valid WASM module, ")?;

        f.write_str(if self.value().is_empty() {
            "missing"
        } else {
            "expected"
        })?;

        write!(
            f,
            " WebAssembly magic \"{}\"",
            crate::module::preamble::MAGIC.escape_ascii()
        )?;

        if !self.value().is_empty() {
            write!(f, ", but got {}", self.value().escape_ascii())?;
        }

        Ok(())
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidMagic {}

/// Parses the [WebAssembly **`magic`** number](MAGIC), indicating the start of a WebAssembly
/// binary format module.
///
/// See also [`parse()`] for parsing the magic number and the **`version`** field that follows.
pub fn magic<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, (), E> {
    nom::bytes::streaming::tag(MAGIC)
        .map(|_| ())
        .with_error_cause(|input| ErrorCause::PreambleMagic(InvalidMagic::new(input)))
        .parse(input)
}

/// Parses a module preamble, checking that the contents of its **`version`** field matches the
/// [`RECOGNIZED_VERSION`].
///
/// To handle different version values, use [`parse_any()`].
pub fn parse<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, (), E> {
    let (input, ()) = magic(input)?;
    nom::combinator::cut(nom::bytes::streaming::tag(RECOGNIZED_VERSION))
        .map(|_| ())
        .with_error_cause(|input| {
            ErrorCause::PreambleVersion(
                input
                    .get(..4)
                    .map(|version| u32::from_le_bytes(version.try_into().unwrap())),
            )
        })
        .parse(input)
}

/// Parses a module preamble, returning the contents of its **`version`** field.
///
/// If you don't want to handle special version values, use [`parse()`] instead.
pub fn parse_any<'a, E: ErrorSource<'a>>(input: &'a [u8]) -> Parsed<'a, [u8; 4], E> {
    let (input, ()) = magic(input)?;
    nom::combinator::cut(nom::bytes::complete::take(4usize))
        .map(|version: &[u8]| version.try_into().unwrap())
        .with_error_cause(|_| ErrorCause::PreambleVersion(None))
        .parse(input)
}
