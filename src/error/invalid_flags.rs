/// Used with [`InvalidFlags`] to indicate what values were invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum InvalidFlagsValue<V: Copy> {
    /// The flags were parsed, but had invalid flags.
    Invalid {
        /// The flags value that *contains* the invalid flags.
        value: V,
        /// The flags that caused validation to fail.
        invalid: V,
    },
    /// Flags could not be parsed.
    Missing,
}

/// Error type used when a byte or 32-bit flags combination was invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InvalidFlags {
    /// Invalid flags for [`Limits`](crate::types::Limits).
    Limits(InvalidFlagsValue<u8>),
    /// Invalid flags for a [`GlobalType`](crate::types::GlobalType).
    GlobalType(InvalidFlagsValue<u8>),
}

impl core::fmt::Display for InvalidFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (name, invalid) = match self {
            Self::Limits(e) => ("limits", e),
            Self::GlobalType(e) => ("global type", e),
        };

        match invalid {
            InvalidFlagsValue::Invalid { value, invalid } => write!(
                f,
                "the {name} flags {value:#04X} contains invalid flag(s): {invalid:#04X}"
            ),
            InvalidFlagsValue::Missing => write!(f, "missing {name} flags"),
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidFlags {}
