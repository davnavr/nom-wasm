use crate::storage::Heap;
use allocator_api2::{alloc::Allocator, vec::Vec};

/// Provides a [`Heap`] implementation over an [`Allocator`].
#[derive(Copy, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "allocator-api2")))]
pub struct AllocatorHeap<A: Clone + Allocator> {
    allocator: A,
}

impl<A: Clone + Allocator> Clone for AllocatorHeap<A> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.allocator.clone())
    }
}

#[allow(missing_docs)]
impl<A: Clone + Allocator> AllocatorHeap<A> {
    pub const fn new(allocator: A) -> Self {
        Self { allocator }
    }

    pub fn into_allocator(self) -> A {
        self.allocator
    }
}

impl<A: Clone + Allocator> Heap for AllocatorHeap<A> {
    type Box<T: ?Sized> = allocator_api2::boxed::Box<T, A>;
    type Vector<T> = Vec<T, A>;

    #[inline]
    fn vector<T>(&self) -> Self::Vector<T> {
        Vec::new_in(self.allocator.clone())
    }

    #[inline]
    fn vector_with_capacity<T>(&self, capacity: usize) -> Self::Vector<T> {
        Vec::with_capacity_in(capacity, self.allocator.clone())
    }
}
