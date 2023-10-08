//! Traits for parsing sequences of items.

use crate::{error::ErrorSource, input::Result};

mod sequence_iter;
mod vector_parser;

pub use sequence_iter::Iter;
pub use vector_parser::VectorParser;

/// Trait for a sequence of items that can be parsed.
///
/// It is recommended that implements of [`Sequence`] also provide an [`IntoIterator`]
/// implementation to convert to a [`sequence::Iter`](Iter).
#[must_use = "items are parsed lazily, call Sequence::next"]
pub trait Sequence<'a, E: ErrorSource<'a>> {
    /// The type of item that is parsed.
    type Item;

    /// Parses the next item in the sequence.
    ///
    /// If all items have already been parsed, or a previous call returned `Some(Err(_))`, then
    /// `None` if returned.
    fn next(&mut self) -> Option<Result<Self::Item, E>>;

    /// Returns an estimate of the remaining number of items in the sequence.
    ///
    /// See the documentation for [`Iterator::size_hint()`] for more information.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

/// Trait for a [`Sequence`] of items with an expected length.
pub trait Vector<'a, E: ErrorSource<'a>>: Sequence<'a, E> {
    /// Returns the expected remaining number of times future calls to
    /// [`Sequence::next()`] returns `Some(Ok(_))`.
    fn expected_len(&self) -> usize;

    /// Returns `true` if the expected remaining number of items is `0`.
    #[inline]
    fn is_empty(&self) -> bool {
        self.expected_len() == 0
    }
}

impl<'a, S, E> Sequence<'a, E> for &mut S
where
    S: Sequence<'a, E> + ?Sized,
    E: ErrorSource<'a>,
{
    type Item = S::Item;

    #[inline]
    fn next(&mut self) -> Option<Result<S::Item, E>> {
        S::next(self)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        S::size_hint(self)
    }
}
