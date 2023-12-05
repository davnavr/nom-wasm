use core::fmt::{Display, Formatter};

/// Describes an [`ErrorCause`] where the length of some data was incorrect.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub struct LengthMismatch {
    pub expected: u32,
    pub actual: u32,
}

impl LengthMismatch {
    fn print(&self, name: &str, f: &mut Formatter) -> core::fmt::Result {
        write!(
            f,
            "expected {} bytes for {name}, but got {}",
            self.expected, self.actual
        )
    }
}

/// Error type used when a byte or 32-bit enumeration value was invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InvalidTag {
    /// An invalid [`ModuleSectionId`](crate::module::ModuleSectionId).
    ModuleSectionId(u8),
    #[allow(missing_docs)]
    FuncType(Option<u8>),
    /// An invalid [`ImportDesc`](crate::module::ImportDesc).
    ImportDesc(Option<u8>),
}

impl Display for InvalidTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let (value, value_width) = match self {
            Self::ModuleSectionId(b) => (Some(u32::from(*b)), 4),
            Self::FuncType(b) | Self::ImportDesc(b) => (b.map(u32::from), 4),
        };

        let name = match self {
            Self::ModuleSectionId(_) => "module section ID",
            Self::FuncType(_) => "function type",
            Self::ImportDesc(_) => "import desc",
        };

        if let Some(value) = value {
            write!(
                f,
                "the {name} tag {value:#0value_width$X} ({value}) is invalid"
            )
        } else {
            write!(f, "missing {name} tag")
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidTag {}

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

impl Display for InvalidFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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

/// Describes why a parser error occured.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ErrorCause {
    Nom(nom::error::ErrorKind),
    Leb128 {
        destination: crate::values::leb128::Destination,
        reason: crate::values::leb128::InvalidEncoding,
    },
    InvalidTag(InvalidTag),
    InvalidFlags(InvalidFlags),
    Vector(crate::values::InvalidVector),
    #[non_exhaustive]
    NameLength,
    NameContents(LengthMismatch),
    NameEncoding(core::str::Utf8Error),
    Index(&'static &'static str),
    #[non_exhaustive]
    SectionId,
    #[non_exhaustive]
    SectionLength,
    SectionContents(LengthMismatch),
    #[non_exhaustive]
    CustomSectionName,
    PreambleMagic(crate::module::preamble::InvalidMagic),
    PreambleVersion(Option<u32>),
    /// A [`BlockType`](crate::types::BlockType) could not be parsed.
    /// - Contains `None` if the end of input was unexpectedly encountered.
    /// - Contains `Some` negative value if an unrecognized encoding for a type was encountered.
    /// - Contains `Some` positive value if the parsed [`TypeIdx`] was too large.
    ///
    /// [`TypeIdx`]: crate::module::TypeIdx
    BlockType(Option<core::num::NonZeroI64>),
    /// A [`ValType`](crate::types::ValType) was not valid.
    ///
    /// - Contains `None` if a [`BlockType::Empty`] was parsed.
    /// - Contains `Some` type index if a [`BlockType::Index`] was parsed.
    ///
    /// [`BlockType::Empty`]: crate::types::BlockType::Empty
    /// [`BlockType::Index`]: crate::types::BlockType::Index
    ValType(Option<crate::module::TypeIdx>),
    Limits {
        index_type: crate::types::IdxType,
        component: LimitsComponent,
    },
    RefType(crate::types::ValType),
    #[non_exhaustive]
    MemType,
    #[non_exhaustive]
    TableType,
    #[non_exhaustive]
    GlobalType,
    #[non_exhaustive]
    TagType,
    #[non_exhaustive]
    ImportDesc {
        kind: u8,
    },
    Import(ImportComponent),
    ModuleSectionOrder(crate::ordering::OrderingError<crate::module::ModuleSectionOrder>),
    Opcode(crate::isa::InvalidOpcode),
    #[non_exhaustive]
    Instr {
        opcode: crate::isa::Opcode,
        reason: crate::isa::InvalidInstr,
    },
    Expr(crate::isa::InvalidExpr),
    MemArg(MemArgComponent),
}

crate::static_assert::check_size!(ErrorCause, <= 16);

impl From<nom::error::ErrorKind> for ErrorCause {
    #[inline]
    fn from(error: nom::error::ErrorKind) -> Self {
        Self::Nom(error)
    }
}

impl ErrorCause {
    /// Attempts to map this [`ErrorCause`] to its closest [`nom::error::ErrorKind`] counterpart.
    ///
    /// This conversion may result in the loss of error information.
    pub fn to_error_kind(self) -> nom::error::ErrorKind {
        use crate::{
            isa::{InvalidExpr, InvalidInstr},
            values::InvalidVector,
        };
        use nom::error::ErrorKind as Kind;

        match self {
            Self::Nom(kind) => kind,
            Self::Leb128 { .. } | Self::Index(_) => Kind::ManyTill,
            Self::InvalidTag(_)
            | Self::PreambleMagic(_)
            | Self::PreambleVersion(_)
            | Self::ImportDesc { .. }
            | Self::Opcode(_) => Kind::Tag,
            Self::InvalidFlags(_) => Kind::OneOf,
            Self::Vector(InvalidVector::Length) | Self::NameLength | Self::SectionLength => {
                Kind::LengthValue
            }
            Self::Vector(InvalidVector::Remaining { .. }) => Kind::Count,
            Self::NameContents(_) | Self::SectionContents(_) => Kind::Complete,
            Self::NameEncoding(_)
            | Self::BlockType(_)
            | Self::ValType(_)
            | Self::RefType(_)
            | Self::ModuleSectionOrder(_)
            | Self::Expr(InvalidExpr::BlockNestingOverflow)
            | Self::Expr(InvalidExpr::ExpectedEnds(_))
            | Self::Instr {
                reason:
                    InvalidInstr::Unrecognized
                    | InvalidInstr::BrTableLabelCount
                    | InvalidInstr::SelectTypedArity(_),
                ..
            } => Kind::Verify,
            Self::SectionId
            | Self::CustomSectionName
            | Self::Limits { .. }
            | Self::MemType
            | Self::TableType
            | Self::GlobalType
            | Self::TagType
            | Self::Import(_)
            | Self::MemArg(_)
            | Self::Instr {
                reason:
                    InvalidInstr::Argument
                    | InvalidInstr::Source
                    | InvalidInstr::Destination
                    | InvalidInstr::VectorLane,
                ..
            } => Kind::Eof,
        }
    }
}

impl Display for ErrorCause {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nom(err) => write!(f, "could not parse: {}", err.description()),
            Self::Leb128 {
                destination,
                reason,
            } => {
                use crate::values::leb128::{Destination, InvalidEncoding};

                f.write_str("could not parse LEB128 encoded ")?;

                match destination {
                    Destination::U32 | Destination::U64 => f.write_str("un")?,
                    Destination::S32 | Destination::S64 => (),
                }

                f.write_str("signed ")?;

                match destination {
                    Destination::U32 | Destination::S32 => f.write_str("32")?,
                    Destination::U64 | Destination::S64 => f.write_str("64")?,
                }

                f.write_str("-bit integer")?;

                match reason {
                    InvalidEncoding::Overflow => {
                        write!(f, ", an overflow occured while decoding the value")
                    }
                    InvalidEncoding::NoContinuation => Ok(()),
                }
            }
            Self::InvalidTag(tag) => Display::fmt(tag, f),
            Self::InvalidFlags(flags) => Display::fmt(flags, f),
            Self::Vector(bad) => Display::fmt(bad, f),
            Self::NameLength => f.write_str("expected name length"),
            Self::NameContents(e) => e.print("UTF-8 encoded name", f),
            Self::NameEncoding(e) => write!(f, "invalid name encoding: {e}"),
            Self::Index(name) => write!(f, "could not parse {name} index"),
            Self::SectionId => f.write_str("missing section ID byte"),
            Self::SectionLength => f.write_str("expected section content length"),
            Self::SectionContents(e) => e.print("section contents", f),
            Self::CustomSectionName => f.write_str("expected custom section name"),
            Self::PreambleMagic(bad) => Display::fmt(bad, f),
            Self::PreambleVersion(None) => f.write_str("missing WASM preamble version"),
            Self::PreambleVersion(Some(actual)) => {
                let expected = u32::from_le_bytes(crate::module::preamble::RECOGNIZED_VERSION);
                write!(f, "expected WASM preamble version {expected} ({expected:#010X}), but got {actual} ({actual:#010X})")
            }
            Self::BlockType(None) => f.write_str("expected valtype, typeidx, or empty block type"),
            Self::BlockType(Some(block_type)) => {
                if block_type.get() < 0 {
                    write!(f, "{block_type} is not a valid value type or block type")
                } else {
                    write!(f, "type index in block type {block_type} is too large, maximum 32-bit value is {}", u32::MAX)
                }
            }
            Self::ValType(None) => f.write_str("expected valtype but got empty block type"),
            Self::ValType(Some(index)) => write!(f, "expected valtype but got type index {index}"),
            Self::Limits {
                index_type,
                component,
            } => {
                f.write_str("could not parse ")?;
                f.write_str(match index_type {
                    crate::types::IdxType::I32 => "32",
                    crate::types::IdxType::I64 => "64",
                })?;
                write!(f, "-bit integer {component} bound for limit")
            }
            Self::RefType(actual) => write!(f, "expected reftype but got {actual}"),
            Self::MemType => f.write_str("could not parse memory type"),
            Self::TableType => f.write_str("could not parse table type"),
            Self::GlobalType => f.write_str("could not parse global type"),
            Self::TagType => f.write_str("could not parse tag type"),
            Self::ImportDesc { kind } => write!(f, "error parsing importdesc kind {kind:#04X}"),
            Self::Import(field) => write!(f, "could not parse import: missing {field}"),
            Self::ModuleSectionOrder(order) => Display::fmt(order, f),
            Self::Opcode(bad) => Display::fmt(bad, f),
            Self::Instr { opcode, reason } => {
                write!(f, "could not parse `{opcode}` instruction {reason}")
            }
            Self::Expr(bad) => Display::fmt(bad, f),
            Self::MemArg(bad) => write!(f, "could not parse memarg: {bad}"),
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for ErrorCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::InvalidTag(e) => e,
            Self::InvalidFlags(e) => e,
            Self::NameEncoding(e) => e,
            Self::PreambleMagic(e) => e,
            Self::ModuleSectionOrder(e) => e,
            Self::Opcode(e) => e,
            Self::Instr { reason, .. } => reason,
            Self::Expr(e) => e,
            _ => return None,
        })
    }
}
