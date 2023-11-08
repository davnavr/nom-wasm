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
    /// A `br_table` instruction had too many labels.
    BrTableLabelCount,
    /// A typed `select` instruction had too many types.
    SelectTypedArity(core::num::NonZeroU8),
}

crate::static_assert::check_size!(InvalidInstr, <= 2);

impl core::fmt::Display for InvalidInstr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unrecognized => f.write_str("instruction was not recognized by this parser"),
            Self::Argument => f.write_str("could not parse immediate argument"),
            Self::Destination => f.write_str("could not parse destination index"),
            Self::Source => f.write_str("could not parse source index"),
            Self::VectorLane => f.write_str("could not parse vector lane index"),
            Self::BrTableLabelCount => {
                f.write_str("`br_table` instruction specified too many labels")
            }
            Self::SelectTypedArity(arity) => {
                f.write_str("`select` instruction should specify at most 1 type")?;
                if core::num::NonZeroU8::MAX == *arity {
                    write!(f, ", but {arity} types were encoded")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidInstr {}
