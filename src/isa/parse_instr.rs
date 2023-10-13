use crate::error::ErrorSource;

/// Error type used by the [`ParseInstr`] trait's methods.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum ParseInstrError<E> {
    /// An immediate argument for the parsed instruction could not be parsed.
    ParseFailed(E),
    /// The [`ParseInstr`] trait does not recognize the instruction that was parsed.
    Unrecognized,
}

impl<E: core::fmt::Display> core::fmt::Display for ParseInstrError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParseFailed(err) => core::fmt::Display::fmt(err, f),
            Self::Unrecognized => f.write_str("instruction was not recognized"),
        }
    }
}

/// Result type used by the [`ParseInstr`] trait's methods.
pub type Result<T, E> = core::result::Result<T, ParseInstrError<E>>;

/// Trait for parsing [WebAssembly instructions].
///
/// [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html
pub trait ParseInstr<'a> {
    /// Error type used when parsing an instruction's immediate argument fails.
    type Error: crate::error::ErrorSource<'a>;

    //fn unreachable(&mut self);
    //fn nop(&mut self);
}
