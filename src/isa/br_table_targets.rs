use crate::{error::ErrorSource, index::IndexVectorParser, input::Result, isa::LabelIdx};

/// Parses the targets of a [`br_table`] instruction as a vector containing **at least one**
/// [`LabelIdx`], with the last label specifying the default target.
///
/// [`br_table`]: crate::isa::ParseInstr::br_table
#[derive(Clone)]
#[must_use]
pub struct BrTableTargets<'a, E: ErrorSource<'a> = crate::error::Error<'a>> {
    targets: IndexVectorParser<'a, LabelIdx, E>,
}

#[allow(missing_docs)]
impl<'a, E: ErrorSource<'a>> BrTableTargets<'a, E> {
    pub fn with_input(input: &'a [u8]) -> Result<Self, E> {
        let (remaining, count) = crate::values::vector_length(input)?;
        if let Some(count) = count.checked_add(1) {
            Ok(Self {
                targets: IndexVectorParser::new(count, remaining, Default::default()),
            })
        } else {
            Err(nom::Err::Failure(E::from_error_kind(
                input,
                nom::error::ErrorKind::Verify,
                //SelectTypedBadArity
            )))
        }
    }

    pub fn finish(self) -> crate::Parsed<'a, (), E> {
        self.targets.finish().map(|(input, _)| (input, ()))
    }
}

impl<'a, E: ErrorSource<'a>> Iterator for BrTableTargets<'a, E> {
    type Item = Result<LabelIdx, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.targets.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.targets.size_hint()
    }
}

impl<'a, E: ErrorSource<'a>> ExactSizeIterator for BrTableTargets<'a, E> {
    #[inline]
    fn len(&self) -> usize {
        self.targets.len()
    }
}

impl<'a, E: ErrorSource<'a>> core::fmt::Debug for BrTableTargets<'a, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BrTableTargets").finish_non_exhaustive()
    }
}
