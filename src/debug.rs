//! Utilities for [`Debug`] formatting.

use core::fmt::Debug;

pub(crate) struct FmtResult<T, E>(pub(crate) Result<T, E>);

impl<T: Debug, E: Debug> Debug for FmtResult<T, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let item: &dyn Debug = match &self.0 {
            Ok(ok) => ok,
            Err(err) => err,
        };

        item.fmt(f)
    }
}
