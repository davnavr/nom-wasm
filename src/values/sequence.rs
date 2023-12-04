//! Contains traits and types for parsing sequences of items.

use crate::input::Result;
use core::fmt::Debug;

/// Trait for parsing sequences of items.
pub trait Sequence<'a>:
    crate::input::AsInput<'a> + Iterator<Item = Result<Self::Output, Self::Error>>
{
    /// The type returned when an items is successfully parsed.
    type Output;

    /// The type returned when an item could not be parsed.
    type Error: crate::error::ErrorSource<'a>;

    /// Parses the next item in the sequence.
    ///
    /// If there are no more items remaining, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if an item could not be parsed.
    #[inline]
    fn parse(&mut self) -> Result<Option<Self::Output>, Self::Error> {
        self.next().transpose()
    }
}

impl<'a, T, E, S> Sequence<'a> for S
where
    E: crate::error::ErrorSource<'a>,
    S: crate::input::AsInput<'a> + Iterator<Item = Result<T, E>>,
{
    type Output = T;
    type Error = E;
}

/// An [`Iterator`] for parsing a [`Sequence`] of items.
///
/// Rather than yielding [`Result`]s, this [`Iterator`] yields `Some` for each `Ok`, then `None` if
/// the end of the [`Sequence`] is reached or when an error occurs.
///
/// Any error that occurs can then be obtained by calling [`SequenceIter::error()`] or
/// [`SequenceIter::finish()`].
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

    /// Finishes parsing all of the remaining items, returning any error that occured.
    pub fn finish(mut self) -> Result<S, S::Error> {
        for _ in &mut self {}
        self.error.map(move |()| self.sequence)
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
    pub fn into_vec(self) -> Result<alloc::vec::Vec<S::Output>, S::Error> {
        self.error?;
        let mut sequence = self.sequence;
        let mut v = alloc::vec::Vec::new();
        match sequence.size_hint() {
            (_, Some(upper)) => v.reserve_exact(upper),
            (lower, None) => v.reserve(lower),
        }

        while let Some(item) = sequence.parse()? {
            v.push(item);
        }

        Ok(v)
    }

    /// Attempts to [`Clone`] the [`Sequence`], or returns `Err` if an error occured while a
    /// previous item in the [`Sequence`] could not be parsed.
    pub fn try_clone(&self) -> core::result::Result<Self, &nom::Err<S::Error>>
    where
        S: Clone,
    {
        self.error
            .as_ref()
            .map(|_| Self::from(self.sequence.clone()))
    }

    fn debug_fmt(mut self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    where
        S::Output: Debug,
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

impl<'a, S: Sequence<'a>> crate::input::AsInput<'a> for SequenceIter<'a, S> {
    #[inline]
    fn as_input(&self) -> &'a [u8] {
        self.sequence.as_input()
    }
}

impl<'a, S: Sequence<'a>> Iterator for &mut SequenceIter<'a, S> {
    type Item = S::Output;

    fn next(&mut self) -> Option<S::Output> {
        if self.error.is_ok() {
            match self.sequence.next()? {
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
        self.sequence.size_hint()
    }
}

impl<'a, S: Sequence<'a>> core::iter::FusedIterator for &mut SequenceIter<'a, S> {}

impl<'a, S: Sequence<'a>> Debug for SequenceIter<'a, S>
where
    S: Clone,
    S::Output: Debug,
    S::Error: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.try_clone() {
            Err(err) => core::fmt::Debug::fmt(err, f),
            Ok(iter) => iter.debug_fmt(f),
        }
    }
}

/// Provides a [`Debug`] implementation for [`Sequence`]s.
pub(crate) struct SequenceDebug<'a, S: Sequence<'a>> {
    sequence: S,
    _marker: core::marker::PhantomData<&'a [u8]>,
}

impl<'a, S> From<S> for SequenceDebug<'a, S>
where
    S: Clone + Sequence<'a>,
    S::Output: Debug,
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
    S::Output: Debug,
    S::Error: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        SequenceIter::from(self.sequence.clone()).debug_fmt(f)
    }
}
