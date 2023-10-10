use crate::types;

/// Indicates the size of indices into a linear memory or table.
///
/// See the [WebAssembly 64-bit memory proposal] for more information.
///
/// [WebAssembly 64-bit memory proposal]: https://github.com/WebAssembly/memory64/tree/main
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum IdxType {
    /// The memory or table is indexed by a 32-bit integer, as it was in the WebAssembly 1.0 release.
    #[default]
    I32,
    /// The memory, is indexed by a 64-bit integer.
    ///
    /// This requires the [*memory64* proposal].
    ///
    /// At the time of writing, no feature proposal introduces 64-bit indices for tables.
    ///
    /// [*memory64* proposal]: https://github.com/WebAssembly/memory64
    I64,
}

impl From<IdxType> for types::NumType {
    #[inline]
    fn from(index_type: IdxType) -> Self {
        match index_type {
            IdxType::I32 => Self::I32,
            IdxType::I64 => Self::I64,
        }
    }
}

impl From<IdxType> for types::ValType {
    #[inline]
    fn from(index_type: IdxType) -> Self {
        types::NumType::from(index_type).into()
    }
}

/// Indicates the minimum size, and an optional maximum size, for the [`Limits`] of a WebAssembly
/// memory or table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum LimitBounds {
    /// The memory or table has 32-bit integer bounds.
    ///
    /// For more information, see the documentation for [`IdxType::I32`].
    #[allow(missing_docs)]
    I32 { min: u32, max: Option<u32> },
    /// The memory has 64-bit integer bounds.
    ///
    /// For more information, see the documentation for [`IdxType::I64`].
    #[allow(missing_docs)]
    I64 { min: u64, max: Option<u64> },
}

impl LimitBounds {
    /// The integer type used to index into the linear memory or table.
    #[inline]
    pub fn index_type(&self) -> IdxType {
        match self {
            Self::I32 { .. } => IdxType::I32,
            Self::I64 { .. } => IdxType::I64,
        }
    }

    /// The minimum size.
    #[inline]
    pub fn minimum(&self) -> u64 {
        match self {
            Self::I32 { min, .. } => u64::from(*min),
            Self::I64 { min, .. } => *min,
        }
    }

    /// The optional maximum size.
    #[inline]
    pub fn maximum(&self) -> Option<u64> {
        match self {
            Self::I32 { max, .. } => max.map(u64::from),
            Self::I64 { max, .. } => *max,
        }
    }
}

impl Default for LimitBounds {
    #[inline]
    fn default() -> Self {
        Self::I32 { min: 0, max: None }
    }
}

/// Indicates whether a linear memory or table is shared, the semantics of which is described in
/// the [WebAssembly threads proposal](https://github.com/WebAssembly/threads).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum Sharing {
    /// The linear memory or table can be used in multiple agents.
    Shared,
    /// The linear memory or table can only be used in a single agent.
    #[default]
    Unshared,
}

/// Describes the minimum and maximum number of pages in a memory or elements in a table.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct Limits {
    /// Declares the minimum and maximum sizes for the corresponding linear memory or table.
    pub bounds: LimitBounds,
    /// Indicates whether or not the corresponding linear memory or table can be used in multiple
    /// agents.
    pub share: Sharing,
}
