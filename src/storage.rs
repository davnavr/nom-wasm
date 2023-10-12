//! Provides traits to support heap allocation.
//!
//! By default, [`nom-wasm`] does not allocate during parsing. This module provides traits to
//! support easier to use APIs for parsing complex WebAssembly structurs (e.g. a [`FuncType`]).
//!
//! If a global allocator is provided, then the standard library types for heap allocation (e.g.
//! [`alloc::vec::Vec`]) can be used.
//!
//! If the `allocator-api2` feature is enabled, then additional trait implementations are provided
//! (e.g. for [`allocator-api2::vec::Vec`]).
//!
//! [`nom-wasm`]: crate
//! [`FuncType`]: crate::types::FuncType

use core::ops::{Deref, DerefMut};

/*
/// Trait for simple heap allocations.
pub trait Box: Deref<Target = Self::Item> + DerefMut {
    /// The item that is pointed to.
    type Item: ?Sized;

    /// Creates a new heap allocation containing the given `item`.
    fn new(item: Self::Item) where Self::Item: Sized;
}
*/

/// Trait for heap allocated arrays that can be resized.
pub trait Vector: Deref<Target = [Self::Item]> + DerefMut {
    /// The item that the vector contains.
    type Item;

    /// Type for the boxed slice version of this vector.
    ///
    /// Used in [`Vector::into_boxed_slice()`].
    type Boxed: Deref<Target = [Self::Item]> + DerefMut<Target = [Self::Item]>;

    /// Converts the vector into a single heap allocation.
    fn into_boxed_slice(self) -> Self::Boxed;

    /// Appends an item to the end of the vector.
    fn push(&mut self, item: Self::Item);

    /// Pops an item from the end of the vector, returning `None` if the vector is empty.
    fn pop(&mut self) -> Option<Self::Item>;

    /// Returns the total number of items that the vector can contain without reallocating.
    fn capacity(&self) -> usize;

    /// Drops all of the items in the vector.
    #[inline]
    fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Reserves space for appending at least `additional` items to the end of the vector.
    #[inline]
    fn reserve(&mut self, additional: usize) {
        let _ = additional;
    }

    /// Reserve space for appending exactly `additional` items to the end of the vector.
    #[inline]
    fn reserve_exact(&mut self, additional: usize) {
        self.reserve(additional)
    }
}

// TODO: macro for implementing Vector

// TODO: impl Vector for ArrayVec

#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[cfg(feature = "alloc")]
impl<T> Vector for alloc::vec::Vec<T> {
    type Item = T;
    type Boxed = alloc::boxed::Box<[T]>;

    #[inline]
    fn into_boxed_slice(self) -> Self::Boxed {
        <Self>::into_boxed_slice(self)
    }

    #[inline]
    fn push(&mut self, item: Self::Item) {
        <Self>::push(self, item);
    }

    #[inline]
    fn pop(&mut self) -> Option<Self::Item> {
        <Self>::pop(self)
    }

    #[inline]
    fn capacity(&self) -> usize {
        <Self>::capacity(self)
    }

    #[inline]
    fn clear(&mut self) {
        <Self>::clear(self);
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        <Self>::reserve(self, additional);
    }

    #[inline]
    fn reserve_exact(&mut self, additional: usize) {
        <Self>::reserve_exact(self, additional);
    }
}
