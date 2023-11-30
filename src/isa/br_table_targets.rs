use crate::error::ErrorSource;

/// Parses the targets of a [`br_table`] instruction as a vector containing **at least one**
/// [`LabelIdx`], with the last label specifying the default target.
///
/// [`br_table`]: crate::isa::ParseInstr::br_table
/// [`LabelIdx`]: crate::isa::LabelIdx
pub struct BrTableTargets<'a, E: ErrorSource<'a> = crate::error::Error<'a>> {
    targets: crate::index::IndexVectorParser<'a, crate::isa::LabelIdx, E>,
}

impl<'a, E: ErrorSource<'a>> BrTableTargets<'a, E> {
    /// Parses the branch table arguments from the given `input`.
    pub fn with_input(input: &'a [u8]) -> crate::input::Result<Self, E> {
        let (remaining, count) = crate::values::vector_length(input)?;
        if let Some(count) = count.checked_add(1) {
            Ok(Self {
                targets: crate::index::IndexVectorParser::new(count, remaining, Default::default()),
            })
        } else {
            Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                nom::error::ErrorKind::Verify,
                crate::error::ErrorCause::Instr {
                    opcode: crate::isa::Opcode::BrTable,
                    reason: crate::isa::InvalidInstr::BrTableLabelCount,
                },
            )))
        }
    }

    #[allow(missing_docs)]
    pub fn finish(self) -> crate::Parsed<'a, (), E> {
        self.targets.into_parser().map(|(input, _)| (input, ()))
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn expected_len(&self) -> usize {
        self.targets.expected_len()
    }
}

impl<'a, E: ErrorSource<'a>> Clone for BrTableTargets<'a, E> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            targets: self.targets.clone(),
        }
    }
}

impl<'a, E: ErrorSource<'a>> crate::input::AsInput<'a> for BrTableTargets<'a, E> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        crate::input::AsInput::as_input(&self.targets)
    }
}

impl<'a, E: ErrorSource<'a>> crate::values::Sequence<'a> for BrTableTargets<'a, E> {
    type Item = crate::isa::LabelIdx;
    type Error = E;

    #[inline]
    fn parse(&mut self) -> crate::input::Result<Option<Self::Item>, Self::Error> {
        crate::values::Sequence::parse(&mut self.targets)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.targets.size_hint()
    }
}

impl<'a, E: ErrorSource<'a> + core::fmt::Debug> core::fmt::Debug for BrTableTargets<'a, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.targets, f)
    }
}
