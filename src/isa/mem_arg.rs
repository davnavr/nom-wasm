use crate::{
    error::{AddCause as _, ErrorCause, MemArgComponent},
    index::Index as _,
    module::MemIdx,
};

/// Specifies the alignment for a [`MemArg`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Align {
    Any,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl Align {
    /// Creates a new alignment value from an exponent of a power of 2.
    pub const fn new(power: u8) -> Option<Self> {
        Some(match power {
            0 => Self::Any,
            1 => Self::Two,
            2 => Self::Four,
            3 => Self::Eight,
            4 => Self::Sixteen,
            _ => return None,
        })
    }

    /// Gets the alignment value, expressed as a number of bytes.
    pub const fn in_bytes(self) -> u8 {
        match self {
            Self::Any => 0,
            Self::Two => 1,
            Self::Four => 4,
            Self::Eight => 8,
            Self::Sixteen => 16,
        }
    }

    /// Gets the alignment value, expressed as the exponent of a power of 2.
    ///
    /// For example, a value of 0 means any alignment, a value of 1 means alignment on a 2-byte
    /// boundary, a value of 3 means alignment on a 4-byte boundary, and so on.
    pub const fn to_power(self) -> u8 {
        match self {
            Self::Any => 0,
            Self::Two => 1,
            Self::Four => 2,
            Self::Eight => 3,
            Self::Sixteen => 4,
        }
    }
}

impl core::fmt::Display for Align {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Any => "1",
            Self::Two => "2",
            Self::Four => "4",
            Self::Eight => "8",
            Self::Sixteen => "16",
        })
    }
}

/// A WebAssembly [**`memarg`**] specifies an address **offset** and expected **align**ment for a
/// load from or store into linear memory.
///
/// [**`memarg`**]: https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct MemArg {
    /// Gets the offset into the linear memory.
    ///
    /// Note that 64-bit offsets require the [`memory64` proposal]. In 32-bit WebAssembly, the
    /// offset is only an [`u32`] value.
    ///
    /// [`memory64` proposal]: https://github.com/WebAssembly/memory64
    pub offset: u64,
    /// The expected alignment for the address of the load or store operation.
    pub align: Align,
    /// The linear memory that the laod or store operation accesses.
    ///
    /// Specifiying a linear memory other than the default (`0`), requires the
    /// [multi-memory proposal](https://github.com/WebAssembly/multi-memory).
    pub memory: MemIdx,
}

impl MemArg {
    /// Parses a [`MemArg`].
    ///
    /// # Errors
    ///
    /// Returns an error if a [`MemArgComponent`] could not be parsed, or if the [**`align`**]
    /// field value is too large.
    ///
    /// [**`align`**]: MemArg::align
    pub fn parse<'a, E>(input: &'a [u8]) -> crate::Parsed<'a, Self, E>
    where
        E: crate::error::ErrorSource<'a>,
    {
        let (input, a) = crate::values::leb128_u32(input)
            .add_cause(ErrorCause::MemArg(MemArgComponent::Alignment(None)))?;

        let (input, offset) = crate::values::leb128_u64(input)
            .add_cause(ErrorCause::MemArg(MemArgComponent::Offset))?;

        let align: u32;

        let (input, memory) = if a < 64 {
            align = a;
            (input, MemIdx(0))
        } else {
            align = a - 64;
            MemIdx::parse(input).add_cause(ErrorCause::MemArg(MemArgComponent::Memory))?
        };

        if let Some(align) = u8::try_from(align).ok().and_then(Align::new) {
            Ok((
                input,
                Self {
                    offset,
                    align,
                    memory,
                },
            ))
        } else {
            Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                nom::error::ErrorKind::Verify,
                ErrorCause::MemArg(MemArgComponent::Alignment(Some(align))),
            )))
        }
    }
}
