/// Indicates why a WebAssembly instruction could not be parsed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InvalidInstr {
    /// The given [`ParseInstr`] implementation indicated that the instruction was [`Unrecognized`].
    ///
    /// [`ParseInstr`]: crate::isa::ParseInstr
    /// [`Unrecognized`]: crate::isa::ParseInstrError::Unrecognized
    Unrecognized,
    /// An immediate argument of an instruction could not be parsed.
    Argument,
    /// The destination index could not be parsed.
    Destination,
    /// The source index could not be parsed.
    Source,
    /// A vector [**`laneidx`**](crate::isa::LaneIdx) could not be parsed.
    VectorLane,
}

impl core::fmt::Display for InvalidInstr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unrecognized => f.write_str("instruction was not recognized by this parser"),
            Self::Argument => f.write_str("could not parse immediate argument"),
            Self::Destination => f.write_str("could not parse destination index"),
            Self::Source => f.write_str("could not parse source index"),
            Self::VectorLane => f.write_str("could not parse vector lane index"),
        }
    }
}
