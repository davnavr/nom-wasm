use crate::{
    error::ErrorSource,
    sequence::{Sequence, Vector},
};

/// Wraps a [`Sequence`] to use it as an [`Iterator`].
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Iter<'a, S, E = crate::error::Error<'a>>
where
    S: Sequence<'a, E>,
    E: ErrorSource<'a>,
{
    sequence: S,
    _marker: core::marker::PhantomData<&'a fn() -> E>,
}

impl<'a, S, E> Iter<'a, S, E>
where
    S: Sequence<'a, E>,
    E: ErrorSource<'a>,
{
    /// Wraps the given [`Sequence`], allowing it to be used as an [`Iterator`].
    #[inline]
    pub fn wrap(sequence: S) -> Self {
        Self {
            sequence,
            _marker: core::marker::PhantomData,
        }
    }

    /// Gets the underlying [`Sequence`].
    #[inline]
    pub fn into_sequence(self) -> S {
        self.sequence
    }
}

impl<'a, S, E> Default for Iter<'a, S, E>
where
    S: Sequence<'a, E> + Default,
    E: ErrorSource<'a>,
{
    #[inline]
    fn default() -> Self {
        Self::wrap(S::default())
    }
}

impl<'a, S, E> Iterator for Iter<'a, S, E>
where
    S: Sequence<'a, E>,
    E: ErrorSource<'a>,
{
    type Item = crate::input::Result<S::Item, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        S::next(&mut self.sequence)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        S::size_hint(&self.sequence)
    }
}

impl<'a, S, E> core::iter::FusedIterator for Iter<'a, S, E>
where
    S: Sequence<'a, E>,
    E: ErrorSource<'a>,
{
}

impl<'a, S, E> core::iter::ExactSizeIterator for Iter<'a, S, E>
where
    S: Vector<'a, E>,
    E: ErrorSource<'a>,
{
    #[inline]
    fn len(&self) -> usize {
        S::expected_len(&self.sequence)
    }
}
