use crate::{
    error::{self, ErrorSource},
    input::Result,
    module::{ModuleSection, ModuleSectionId},
    ordering::Ordering,
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

/// Represents either a [`ModuleSection`] or a section with an unknown [*id*]
///
/// [*id*]: Section::id
#[derive(Clone)]
pub struct UnknownModuleSection<'a> {
    // Non-public fields, since they may be changed (e.g. could get ModuleSection from Section)
    remaining: &'a [u8],
    section_id: u8,
    section_contents: &'a [u8],
    known: Option<ModuleSection<'a>>,
    ordering: Ordering<ModuleSectionOrder>,
}

impl<'a> UnknownModuleSection<'a> {
    fn new<E: ErrorSource<'a>>(
        remaining: &'a [u8],
        section: Section<'a>,
        ordering: &mut Ordering<ModuleSectionOrder>,
    ) -> Result<Self, E> {
        let saved_ordering = ordering.clone();
        let known = match ModuleSection::interpret_section(&section) {
            Ok(result) => {
                let module_section = result?;

                if let Some(next) = ModuleSectionOrder::from_section_id(module_section.id()) {
                    ordering.check(next).map_err(|e| {
                        nom::Err::Failure(E::from_error_kind_and_cause(
                            remaining,
                            error::ErrorKind::Verify,
                            error::ErrorCause::ModuleSectionOrder(e),
                        ))
                    })?;
                }

                Some(module_section)
            }
            Err(_) => None,
        };

        Ok(Self {
            remaining,
            section_id: section.id,
            section_contents: section.contents,
            known,
            ordering: saved_ordering,
        })
    }

    /// The remaining input, starting with the [*id*] of this module section.
    ///
    /// [*id*]: Section::id
    #[inline]
    pub fn remaining_input(&self) -> &'a [u8] {
        self.remaining
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn section(&self) -> Section<'a> {
        Section {
            id: self.section_id,
            contents: self.section_contents,
        }
    }

    /// Interprets the [`Section`] as a [`ModuleSection`].
    ///
    /// Returns `None` if the section was neither a known module section or a [`Custom`] section.
    ///
    /// See the documentation for [`ModuleSection::interpret_section()`] for more information.
    ///
    /// [`Custom`]: ModuleSection::Custom
    #[inline]
    pub fn to_module_section(&self) -> Option<ModuleSection<'a>> {
        self.known.clone()
    }

    /// The current [`ModuleSectionOrder`] when this module section was parsed.
    ///
    /// Call the [`Ordering::previous()`] method to determine which [`ModuleSection`] was
    /// previously parsed.
    #[inline]
    pub fn ordering(&self) -> Ordering<ModuleSectionOrder> {
        self.ordering.clone()
    }
}

impl core::fmt::Debug for UnknownModuleSection<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut s;
        if let Some(section) = &self.known {
            s = f.debug_struct("Known");
            s.field("section", section)
        } else {
            s = f.debug_struct("Unknown");
            s.field("section", &self.section())
        }
        .field("ordering", &self.ordering)
        .finish()
    }
}

/// Parses the sequence of [`ModuleSection`]s after the [`preamble`] within a WebAssembly module.
///
/// An error is yielded as the last item if a [`Section`] could not be parsed, or if non-custom [`ModuleSection`]s were not in the
/// correct order according to the [`ModuleSectionOrder`].
///
/// [`preamble`]: crate::module::preamble
#[derive(Default)]
#[must_use = "call Iterator::next() or SequenceIter::finish()"]
pub struct ModuleSectionSequence<'a, E: ErrorSource<'a>> {
    sections: crate::section::Sequence<'a, E>,
    ordering: Ordering<ModuleSectionOrder>,
}

impl<'a, E> From<crate::section::Sequence<'a, E>> for ModuleSectionSequence<'a, E>
where
    E: ErrorSource<'a>,
{
    fn from(sections: crate::section::Sequence<'a, E>) -> Self {
        Self {
            sections,
            ordering: Ordering::new(),
        }
    }
}

impl<'a, E> From<ModuleSectionSequence<'a, E>> for crate::section::Sequence<'a, E>
where
    E: ErrorSource<'a>,
{
    #[inline]
    fn from(module_sections: ModuleSectionSequence<'a, E>) -> Self {
        module_sections.sections
    }
}

impl<'a, E: ErrorSource<'a>> Clone for ModuleSectionSequence<'a, E> {
    fn clone(&self) -> Self {
        Self {
            sections: self.sections.clone(),
            ordering: self.ordering.clone(),
        }
    }
}

impl<'a, E: ErrorSource<'a>> ModuleSectionSequence<'a, E> {
    /// Creates a [`ModuleSectionSequence`] from the sections contained within the given `input`.
    #[inline]
    pub fn new(input: &'a [u8]) -> Self {
        crate::section::Sequence::new(input).into()
    }

    /// Gets the current ordering of [`ModuleSection`]s.
    #[inline]
    pub fn ordering(&self) -> Ordering<ModuleSectionOrder> {
        self.ordering.clone()
    }

    // TODO: Have a MapSequence struct?
    // /// Returns an [`Iterator`] that returns an [`Err`] for unknown [`Section`]s.
    // ///
    // /// An error is yielded if a [`Section`] could not be parsed, if non-custom a
    // /// [`ModuleSection`]s were not in the correct order, or if a non-custom [`Section`] with
    // /// an unknown [*id*] was encountered.
    // pub fn without_unknown(
    //     self,
    // ) -> impl Iterator<Item = Result<(ModuleSection<'a>, Ordering<ModuleSectionOrder>), E>> {
    //     self.map(|result| {
    //         let section = result?;
    //         if let Some(known) = section.to_module_section() {
    //             Ok((known, section.ordering()))
    //         } else {
    //             Err(nom::Err::Failure(E::from_error_kind_and_cause(
    //                 section.remaining_input(),
    //                 error::ErrorKind::Verify,
    //                 error::ErrorCause::InvalidTag(error::InvalidTag::ModuleSectionId(
    //                     section.section.id,
    //                 )),
    //             )))
    //         }
    //     })
    // }
}

impl<'a, E: ErrorSource<'a>> crate::input::AsInput<'a> for ModuleSectionSequence<'a, E> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        crate::input::AsInput::as_input(&self.sections)
    }
}

impl<'a, E: ErrorSource<'a>> crate::values::Sequence<'a> for ModuleSectionSequence<'a, E> {
    type Item = UnknownModuleSection<'a>;
    type Error = E;

    /// Parses the next [`ModuleSection`] or unknown [`Section`].
    ///
    /// # Errors
    ///
    /// Returns an error if a [`Section`] could not be parsed, or if non-custom [`ModuleSection`]s
    /// were not in the correct order.
    fn parse(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        // ModuleSectionOrder::from_section_id(known.id())

        // order.check(next).map_err(|e| {
        //     nom::Err::Failure(E::from_error_kind_and_cause(
        //         input,
        //         error::ErrorKind::Verify,
        //         error::ErrorCause::ModuleSectionOrder(e),
        //     ))
        // })?;

        match self.sections.parse() {
            Ok(None) => Ok(None),
            Err(err) => Err(err),
            Ok(Some((remaining, section))) => Ok(Some(UnknownModuleSection::new(
                remaining,
                section,
                &mut self.ordering,
            )?)),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.sections.size_hint()
    }
}

impl<'a, E> core::fmt::Debug for ModuleSectionSequence<'a, E>
where
    E: ErrorSource<'a> + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&crate::values::SequenceDebug::from(self.clone()), f)
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
#[deprecated(note = "use ModuleSectionSequence")]
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
    // TODO: Make an iterator struct ModuleSectionSequence
    let mut order = crate::ordering::Ordering::<ModuleSectionOrder>::new();
    // crate::section::Sequence::new(input).try_for_each(move |result| {
    //     let (input, section) = result?;
    //     match ModuleSection::interpret_section(&section) {
    //         Ok(result) => {
    //             let known = result?;

    //             if let Some(next) = ModuleSectionOrder::from_section_id(known.id()) {
    //                 order.check(next).map_err(|e| {
    //                     nom::Err::Failure(E::from_error_kind_and_cause(
    //                         input,
    //                         error::ErrorKind::Verify,
    //                         error::ErrorCause::ModuleSectionOrder(e),
    //                     ))
    //                 })?;
    //             }

    //             f(known, *order.previous())
    //         }
    //         Err(_) => g(input, section, *order.previous()),
    //     }
    // })
    todo!()
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
