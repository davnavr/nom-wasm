//! Contains traits, types, and functions for parsing sequences of items.

use crate::input::Result;
use core::fmt::Debug;

/// Trait for parsing sequences of items.
#[must_use = "sequences are lazy and do not parse all items immediately"]
pub trait Sequence<'a>: crate::input::AsInput<'a> {
    /// The type returned when an items is successfully parsed.
    type Item;

    /// The type returned when an item could not be parsed.
    type Error: crate::error::ErrorSource<'a>;

    /// Parses the next item in the sequence.
    ///
    /// If there are no more items remaining, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if an item could not be parsed.
    fn parse(&mut self) -> Result<Option<Self::Item>, Self::Error>;

    /// Returns an estimate of the remaining number of items in the sequence that have yet to be
    /// parsed.
    ///
    /// See the documentation for [`Iterator::size_hint()`].
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    /// Attempts to collect all of the remaining items into a [`Vec`].
    ///
    /// # Errors
    ///
    /// If an item could not be parsed, returns the corresponding error.
    ///
    /// [`Vec`]: alloc::vec::Vec
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[cfg(feature = "alloc")]
    fn into_vec(mut self) -> Result<alloc::vec::Vec<Self::Item>, Self::Error>
    where
        Self: Sized,
    {
        let mut v = alloc::vec::Vec::new();
        match self.size_hint() {
            (_, Some(upper)) => v.reserve_exact(upper),
            (lower, None) => v.reserve(lower),
        }

        while let Some(item) = self.parse()? {
            v.push(item);
        }

        Ok(v)
    }
}

crate::static_assert::object_safe!(Sequence<'static, Item = (), Error = ()>);

impl<'a, S: Sequence<'a>> Sequence<'a> for &mut S {
    type Item = S::Item;
    type Error = S::Error;

    #[inline]
    fn parse(&mut self) -> Result<Option<S::Item>, S::Error> {
        S::parse(self)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        S::size_hint(self)
    }
}

/// Provides an [`Iterator`] implementation for a [`Sequence`].
#[derive(Clone)]
pub struct SequenceIter<'a, S: Sequence<'a>> {
    sequence: S,
    error: Result<(), S::Error>,
}

impl<'a, S: Sequence<'a>> From<S> for SequenceIter<'a, S> {
    #[inline]
    fn from(sequence: S) -> Self {
        Self {
            sequence,
            error: Ok(()),
        }
    }
}

impl<'a, S: Sequence<'a> + Default> Default for SequenceIter<'a, S> {
    #[inline]
    fn default() -> Self {
        S::default().into()
    }
}

impl<'a, S: Sequence<'a>> SequenceIter<'a, S> {
    /// Gets the error that previously occured while parsing the item, if it exists.
    #[inline]
    pub fn error(&self) -> Option<&nom::Err<S::Error>> {
        self.error.as_ref().err()
    }

    /// Gets a reference to the underlying [`Sequence`].
    #[inline]
    pub(crate) fn sequence(&self) -> &S {
        &self.sequence
    }

    /// Finishes parsing all of the remaining items, returning any error that occured.
    pub fn finish(mut self) -> Result<S, S::Error> {
        for _ in &mut self {}
        self.error.map(move |()| self.sequence)
    }

    fn debug_fmt(mut self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    where
        S: Clone,
        S::Item: Debug,
        S::Error: Debug,
    {
        let mut list = f.debug_list();
        list.entries(&mut self);

        if let Err(err) = self.error {
            list.entry(&err);
        }

        list.finish()
    }
}

impl<'a, S: Sequence<'a>> Iterator for &mut SequenceIter<'a, S> {
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        if self.error.is_ok() {
            match self.sequence.parse().transpose()? {
                Ok(item) => Some(item),
                Err(err) => {
                    self.error = Err(err);
                    None
                }
            }
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        S::size_hint(&self.sequence)
    }
}

impl<'a, S: Sequence<'a>> core::iter::FusedIterator for &mut SequenceIter<'a, S> {}

impl<'a, S: Sequence<'a>> Debug for SequenceIter<'a, S>
where
    S: Clone,
    S::Item: Debug,
    S::Error: Clone + Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.clone().debug_fmt(f)
    }
}

/// Provides a [`Debug`] implementation for [`Sequence`]s.
pub(crate) struct SequenceDebug<'a, S> {
    sequence: S,
    _marker: core::marker::PhantomData<&'a [u8]>,
}

impl<'a, S> From<S> for SequenceDebug<'a, S>
where
    S: Clone + Sequence<'a>,
    S::Item: Debug,
    S::Error: Debug,
{
    #[inline]
    fn from(sequence: S) -> Self {
        Self {
            sequence,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'a, S> Debug for SequenceDebug<'a, S>
where
    S: Clone + Sequence<'a>,
    S::Item: Debug,
    S::Error: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        SequenceIter::from(self.sequence.clone()).debug_fmt(f)
    }
}
