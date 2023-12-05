/// Describes an [`ErrorCause`] where the length of some data was incorrect.
///
/// [`ErrorCause`]: crate::error::ErrorCause
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub struct LengthMismatch {
    pub expected: u32,
    pub actual: u32,
}

impl LengthMismatch {
    pub(in crate::error) fn print(
        &self,
        name: &str,
        f: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        write!(
            f,
            "expected {} bytes for {name}, but got {}",
            self.expected, self.actual
        )
    }
}

impl core::fmt::Display for LengthMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "expected {} bytes but got {}",
            self.expected, self.actual
        )
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for LengthMismatch {}
