/// Indicates why a WebAssembly instruction could not be parsed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InvalidInstr {
    /// The given [`ParseInstr`] implementation indicated that the instruction was [`Unrecognized`].
    ///
    /// [`ParseInstr`]: crate::isa::ParseInstr
    /// [`Unrecognized`]: crate::isa::ParseInstrError::Unrecognized
    Unrecognized,
}

impl core::fmt::Display for InvalidInstr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unrecognized => f.write_str("instruction was not recognized by this parser"),
        }
    }
}
