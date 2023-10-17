use crate::storage::Heap;
use alloc::vec::Vec;

/// A [`Heap`] implementation that defers to the Rust standard library's global allocator.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub struct DefaultHeap;

impl Heap for DefaultHeap {
    type Vector<T> = Vec<T>;

    #[inline]
    fn vector_with_capacity<T>(&self, capacity: usize) -> Self::Vector<T> {
        Vec::with_capacity(capacity)
    }

    #[inline]
    fn vector<T>(&self) -> Self::Vector<T> {
        Vec::new()
    }
}
