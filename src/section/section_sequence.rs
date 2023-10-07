use crate::{error::ErrorSource, input, section::Section, sequence};

/// Represents a sequence of WebAssembly [`Section`]s.
#[derive(Clone, Default)]
pub struct SectionSequence<'a> {
    input: &'a [u8],
}

impl<'a> From<&'a [u8]> for SectionSequence<'a> {
    #[inline]
    fn from(input: &'a [u8]) -> Self {
        Self { input }
    }
}

impl<'a, E: ErrorSource<'a>> sequence::Sequence<'a, E> for SectionSequence<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<crate::input::Result<Self::Item, E>> {
        if self.input.is_empty() {
            None
        } else {
            Some(input::parse_with(&mut self.input, Section::parse))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            !self.input.is_empty() as usize,
            Some((self.input.len() + 1) / 2),
        )
    }
}

impl<'a> input::AsInput<'a> for SectionSequence<'a> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.input
    }
}

impl core::fmt::Debug for SectionSequence<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(sequence::Iter::<'_, _>::wrap(self.clone()))
            .finish()
    }
}
