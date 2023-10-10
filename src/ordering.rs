//! The [`Ordering<T>`] struct checks that something is in ascending order.

use core::cmp::PartialOrd;
use core::fmt::Display;

/// Error when the items processed by an [`Ordering<T>`] are not in the correct order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OrderingError<T: Copy> {
    /// A duplicate item was encountered.
    Duplicate(T),
    /// An item was not in the correct order.
    #[allow(missing_docs)]
    OutOfOrder { next: T, previous: T },
}

impl<T: Copy + Display> Display for OrderingError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Duplicate(item) => write!(f, "duplicate {item}"),
            Self::OutOfOrder { next, previous } => {
                write!(f, "incorrect order: {next} should come after {previous}")
            }
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl<T: Copy + core::fmt::Debug + Display> std::error::Error for OrderingError<T> {}

/// Helper struct to ensure that items are in **ascending** order.
///
/// If you need to check for **descending** order instead, use [`Reverse<T>`](core::cmp::Reverse)
#[derive(Clone, Copy, Debug)]
pub struct Ordering<T: Copy + PartialOrd> {
    previous: Option<T>,
}

impl<T: Copy + PartialOrd> Default for Ordering<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + PartialOrd> Ordering<T> {
    /// Constructs a new [`Ordering<T>`].
    #[inline]
    pub const fn new() -> Self {
        Self { previous: None }
    }

    /// Gets the previous value passed to [`check()`](Ordering::check).
    #[inline]
    pub fn previous(&self) -> Option<&T> {
        self.previous.as_ref()
    }

    /// Checks that the `next` item is in the correct order.
    pub fn check(&mut self, next: T) -> Result<(), OrderingError<T>> {
        match self.previous {
            Some(previous) if next <= previous => Err(if next == previous {
                OrderingError::Duplicate(next)
            } else {
                OrderingError::OutOfOrder { next, previous }
            }),
            _ => {
                self.previous = Some(next);
                Ok(())
            }
        }
    }
}
