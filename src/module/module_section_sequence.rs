use crate::{
    error::{self, ErrorSource},
    input::Result,
    module::{ModuleSection, ModuleSectionId},
    section::Section,
};

/// Defines the ordering of [`ModuleSection`]s within a WebAssembly module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ModuleSectionOrder {
    Type,
    Import,
    Func,
    Table,
    Mem,
    Tag,
    Global,
    Export,
    Start,
    Elem,
    DataCount,
    Code,
    Data,
}

impl ModuleSectionOrder {
    const fn from_section_id(id: ModuleSectionId) -> Option<Self> {
        Some(match id {
            ModuleSectionId::Custom => return None,
            ModuleSectionId::Type => Self::Type,
            ModuleSectionId::Import => Self::Import,
        })
    }
}

impl core::fmt::Display for ModuleSectionOrder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Type => "type",
            Self::Import => "import",
            Self::Func => "function",
            Self::Table => "table",
            Self::Mem => "memory",
            Self::Tag => "tag",
            Self::Global => "global",
            Self::Export => "export",
            Self::Start => "start",
            Self::Elem => "elem",
            Self::DataCount => "data count",
            Self::Code => "code",
            Self::Data => "data",
        })?;
        f.write_str(" section")
    }
}

/// Parses the sequence of [`ModuleSection`]s after the [`preamble`] within a WebAssembly module,
/// with custom handling for unknown sections.
///
/// Each [`ModuleSection`] is passed into the `f` closure, and each unknown non-custom [`Section`]
/// into the `g` closure.
///
/// # Errors
///
/// Returns an error if a section could not be parsed, or if non-custom sections were not in the
/// correct order.
///
/// [`preamble`]: crate::module::preamble
pub fn module_section_sequence_with_unknown<'a, E, F, G>(
    input: &'a [u8],
    mut f: F,
    mut g: G,
) -> Result<(), E>
where
    E: ErrorSource<'a>,
    F: FnMut(ModuleSection<'a>, Option<ModuleSectionOrder>) -> Result<(), E>,
    G: FnMut(&'a [u8], Section<'a>, Option<ModuleSectionOrder>) -> Result<(), E>,
{
    let mut order = crate::ordering::Ordering::new();
    crate::section::sequence(
        input,
        |input, section| match ModuleSection::interpret_section(&section) {
            Ok(result) => {
                let known = result?;
                if let Some(next) = ModuleSectionOrder::from_section_id(known.id()) {
                    order.check(next).map_err(|e| {
                        nom::Err::Failure(E::from_error_kind_and_cause(
                            input,
                            error::ErrorKind::Verify,
                            error::ErrorCause::ModuleSectionOrder(e),
                        ))
                    })?;
                }
                f(known, *order.previous())
            }
            Err(_) => g(input, section, *order.previous()),
        },
    )
}

fn no_unknown_section<'a, E: ErrorSource<'a>>(
    input: &'a [u8],
    section: Section<'a>,
    _: Option<ModuleSectionOrder>,
) -> Result<(), E> {
    Err(nom::Err::Failure(E::from_error_kind_and_cause(
        input,
        error::ErrorKind::Verify,
        error::ErrorCause::InvalidTag(error::InvalidTag::ModuleSectionId(section.id)),
    )))
}

/// Parses the sequence of [`ModuleSection`]s after the [`preamble`] within a WebAssembly module.
///
/// To handle unknown non-custom [`Section`]s, use [`module_section_sequence_with_unknown()`] instead.
///
/// # Errors
///
/// Returns an error if a section could not be parsed, if non-custom sections were not in the
/// correct order, or if a non-custom [`Section`] with an unknown [*id*] was encountered.
///
/// [`preamble`]: crate::module::preamble
/// [*id*]: Section::id
pub fn module_section_sequence<'a, E, F>(input: &'a [u8], f: F) -> Result<(), E>
where
    E: ErrorSource<'a>,
    F: FnMut(ModuleSection<'a>, Option<ModuleSectionOrder>) -> Result<(), E>,
{
    module_section_sequence_with_unknown(input, f, no_unknown_section::<'a, E>)
}
