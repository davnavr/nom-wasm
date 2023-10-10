use crate::error;
use nom::ToUsize;

mod import;
mod import_desc;

pub use import::Import;
pub use import_desc::ImportDesc;

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
    /// The expected number of `import`s within the section.
    #[inline]
    pub fn count(&self) -> usize {
        self.count.to_usize()
    }

    /// Parses each `import` within the section, passing them to the given closure.
    pub fn parse_contents<E, F>(&self, mut f: F) -> crate::Parsed<'a, (), E>
    where
        E: error::ErrorSource<'a>,
        F: FnMut(Import<'a>),
    {
        crate::values::sequence(self.imports, self.count(), |input| {
            let (input, import) = Import::parse(input)?;
            f(import);
            Ok((input, ()))
        })
    }

    //pub fn iter_contents(&self) -> IterImportSec // or crate::values::IterVector<'a, E, ImportParser>
}

impl<'a> crate::input::AsInput<'a> for ImportSec<'a> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.imports
    }
}

impl core::fmt::Debug for ImportSec<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: Use iter_contents?
        let mut list = f.debug_list();
        let result = self.parse_contents::<error::Error, _>(|import| {
            list.entry(&import);
        });

        if let Err(err) = result {
            list.entry(&err);
        }

        list.finish()
    }
}
