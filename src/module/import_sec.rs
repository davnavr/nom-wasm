use crate::{error::ErrorSource, input::Result};

mod import;
mod import_desc;

pub use import::{Import, ImportParser};
pub use import_desc::ImportDesc;

/// Iterates over the contents of the [`ImportSec`].
///
/// See the docuemntation for [`ImportSec::iter_contents()`] for more information.
pub type ImportSecIter<'a, E> = crate::values::FullVector<'a, Import<'a>, E, ImportParser>;

/// Represents the [*import section*].
///
/// This corresponds to the [**imports** component] of a WebAssembly module.
///
/// [*import section*]: https://webassembly.github.io/spec/core/binary/modules.html#import-section
/// [**imports** component]: https://webassembly.github.io/spec/core/syntax/modules.html#imports
#[derive(Clone, Copy, Default)]
#[must_use]
pub struct ImportSec<'a> {
    count: u32,
    imports: &'a [u8],
}

impl<'a> ImportSec<'a> {
    /// Parses an [`Import`] section from a section's contents.
    pub fn parse<E: ErrorSource<'a>>(contents: &'a [u8]) -> Result<Self, E> {
        let (imports, count) = crate::values::vector_length(contents)?;
        Ok(Self { count, imports })
    }

    /// The expected number of [`Import`]s within the section.
    #[inline]
    pub fn count(&self) -> usize {
        nom::ToUsize::to_usize(&self.count)
    }

    /// Returns an [`Iterator`] over the [`Import`]s within the section.
    #[inline]
    pub fn iter_contents<E: ErrorSource<'a>>(&self) -> ImportSecIter<'a, E> {
        crate::values::Vector::new(self.count, self.imports, ImportParser).into()
    }
}

impl<'a> crate::input::AsInput<'a> for ImportSec<'a> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.imports
    }
}

impl core::fmt::Debug for ImportSec<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.iter_contents::<crate::error::Error>(), f)
    }
}
