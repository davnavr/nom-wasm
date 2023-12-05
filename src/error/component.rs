use core::fmt::{Display, Formatter};

/// Indicates which part of a [`Limits`](crate::types::Limits) could not be parsed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum LimitsComponent {
    Minimum,
    Maximum,
}

impl Display for LimitsComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
        })
    }
}

/// Indicates which field of an [`Import`](crate::module::Import) could not be parsed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ImportComponent {
    Module,
    Name,
}

impl Display for ImportComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Module => "module name",
            Self::Name => "import name",
        })
    }
}

/// Indicates why a [`MemArg`](crate::isa::MemArg) could not be parsed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum MemArgComponent {
    /// Indicates that the [**`align`**] field could not be parsed when [`None`]; otherwise,
    /// indicates that the [**`align`**] was too large.
    ///
    /// [**`align`**]: crate::isa::MemArg::align
    Alignment(Option<u32>),
    Offset,
    Memory,
}

impl Display for MemArgComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Alignment(Some(a)) => {
                write!(f, "specified alignment was 2^{a}, which is too large")
            }
            Self::Alignment(None) => f.write_str("alignment field"),
            Self::Offset => f.write_str("offset field"),
            Self::Memory => f.write_str("memory field"),
        }
    }
}
