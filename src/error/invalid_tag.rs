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

impl core::fmt::Display for InvalidTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
