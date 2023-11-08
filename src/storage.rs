//! Provides traits to support heap allocation.
//!
//! By default, [`nom-wasm`] does not allocate during parsing. This module provides traits to
//! support easier to use APIs for parsing complex WebAssembly structurs (e.g. a [`FuncType`]).
//!
//! If a global allocator is provided (by enabling the `alloc` feature), then the standard library
//! types for heap allocation (e.g. [`alloc::vec::Vec`]) can be used.
//!
//! If the `allocator-api2` feature is enabled, then additional trait implementations are provided
//! (e.g. for [`allocator_api2::vec::Vec`]).
//!
//! [`nom-wasm`]: crate
//! [`FuncType`]: crate::types::FuncType

/*
/// Trait for simple heap allocations.
pub trait Box: Deref<Target = Self::Item> + DerefMut {
    /// The item that is pointed to.
    type Item: ?Sized;

    /// Creates a new heap allocation containing the given `item`.
    fn new(item: Self::Item) where Self::Item: Sized;
}
*/

#[cfg(feature = "allocator-api2")]
mod allocator_heap;
#[cfg(feature = "alloc")]
mod default_heap;
mod vector;

#[cfg(feature = "allocator-api2")]
pub use allocator_heap::AllocatorHeap;
#[cfg(feature = "alloc")]
pub use default_heap::DefaultHeap;
pub use vector::Vector;

/// Trait that provides associated types and methods for heap allocations.
pub trait Heap {
    /// Type for simple heap allocations.
    type Box<T: ?Sized>: core::ops::Deref<Target = T> + core::ops::DerefMut<Target = T>;

    /// Type used for resizable arrays allocated in this [`Heap`].
    type Vector<T>: Vector<Item = T, Boxed = Self::Box<[T]>>;

    /// Returns an empty [`Vector`].
    #[inline]
    fn vector<T>(&self) -> Self::Vector<T> {
        self.vector_with_capacity(0)
    }

    /// Allocates a new [`Vector`] with sufficient space to contain `capacity` elements.
    fn vector_with_capacity<T>(&self, capacity: usize) -> Self::Vector<T>;
}

impl<H: Heap> Heap for &H {
    type Box<T: ?Sized> = H::Box<T>;
    type Vector<T> = H::Vector<T>;

    #[inline]
    fn vector<T>(&self) -> Self::Vector<T> {
        H::vector(self)
    }

    #[inline]
    fn vector_with_capacity<T>(&self, capacity: usize) -> Self::Vector<T> {
        H::vector_with_capacity(self, capacity)
    }
}
