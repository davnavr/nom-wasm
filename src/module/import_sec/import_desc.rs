use crate::types;

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
    pub fn parse<'a, E: crate::error::ErrorSource<'a>>(
        input: &'a [u8],
    ) -> crate::Parsed<'a, Self, E> {
        use crate::{
            error::{ErrorCause, InvalidTag},
            index::Index as _,
            parser::Parser as _,
        };
        use nom::Parser as _;

        let (input, tag) = if let Some((first, remaining)) = input.split_first() {
            (remaining, *first)
        } else {
            return Err(nom::Err::Failure(E::from_error_cause(
                input,
                ErrorCause::InvalidTag(InvalidTag::ImportDesc(None)),
            )));
        };

        let bad_desc = move |input| (input, ErrorCause::ImportDesc { kind: tag });

        match tag {
            0 => types::TypeIdx::parse
                .map(Self::Function)
                .with_error_cause(bad_desc)
                .parse(input),
            1 => types::TableType::parse
                .map(Self::Table)
                .with_error_cause(bad_desc)
                .parse(input),
            2 => types::MemType::parse
                .map(Self::Memory)
                .with_error_cause(bad_desc)
                .parse(input),
            3 => types::GlobalType::parse
                .map(Self::Global)
                .with_error_cause(bad_desc)
                .parse(input),
            4 => types::TagType::parse
                .map(Self::Tag)
                .with_error_cause(bad_desc)
                .parse(input),
            _ => Err(nom::Err::Failure(E::from_error_cause(
                &input[..1],
                ErrorCause::InvalidTag(InvalidTag::ImportDesc(Some(tag))),
            ))),
        }
    }
}
