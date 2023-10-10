use crate::{
    error::{self, AddCause as _, ErrorCause},
    index::Index as _,
    types,
};

/// An [**`importdesc`**] describes what kind of entity is specified by an [`Import`].
///
/// [**`importdesc`**]: https://webassembly.github.io/spec/core/binary/modules.html#import-section
/// [`Import`]: crate::module::Import
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ImportDesc {
    /// An imported function with the specified signature.
    Function(types::TypeIdx),
    /// An imported table with the specified limits and element type.
    Table(types::TableType),
    /// An imported table with the specified limits.
    Memory(types::MemType),
    /// An imported global with the specified type.
    Global(types::GlobalType),
    /// An imported tag, introduced as part of the [exception handling proposal].
    ///
    /// [exception handling proposal]: https://github.com/WebAssembly/exception-handling/tree/main
    Tag(types::TagType),
}

impl ImportDesc {
    #[allow(missing_docs)]
    pub fn parse<'a, E: error::ErrorSource<'a>>(input: &'a [u8]) -> crate::Parsed<'a, Self, E> {
        let (input, tag) = if let Some((first, remaining)) = input.split_first() {
            (remaining, *first)
        } else {
            return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                error::ErrorKind::OneOf,
                ErrorCause::InvalidTag(error::InvalidTag::ImportDesc(None)),
            )));
        };

        let bad_desc = move || ErrorCause::ImportDesc { kind: tag };

        match tag {
            0 => types::TypeIdx::parse(input)
                .add_cause_with(bad_desc)
                .map(|(input, index)| (input, Self::Function(index))),
            1 => types::TableType::parse(input)
                .add_cause_with(bad_desc)
                .map(|(input, ty)| (input, Self::Table(ty))),
            2 => types::MemType::parse(input)
                .add_cause_with(bad_desc)
                .map(|(input, ty)| (input, Self::Memory(ty))),
            3 => types::GlobalType::parse(input)
                .add_cause_with(bad_desc)
                .map(|(input, ty)| (input, Self::Global(ty))),
            4 => types::TagType::parse(input)
                .add_cause_with(bad_desc)
                .map(|(input, ty)| (input, Self::Tag(ty))),
            _ => Err(nom::Err::Failure(E::from_error_kind_and_cause(
                &input[..1],
                error::ErrorKind::OneOf,
                ErrorCause::InvalidTag(error::InvalidTag::ImportDesc(Some(tag))),
            ))),
        }
    }
}
