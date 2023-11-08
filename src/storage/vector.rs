use core::ops::{Deref, DerefMut};

/// Trait for heap allocated arrays that can be resized.
///
/// This trait is [object safe].
///
/// [object safe]: https://doc.rust-lang.org/beta/reference/items/traits.html#object-safety
pub trait Vector: Deref<Target = [Self::Item]> + DerefMut {
    /// The item that the vector contains.
    type Item;

    /// Type for the boxed slice version of this vector.
    ///
    /// Used in [`Vector::into_boxed_slice()`].
    type Boxed: Deref<Target = [Self::Item]> + DerefMut<Target = [Self::Item]>
    where
        Self: Sized;

    /// Converts the vector into a single heap allocation.
    fn into_boxed_slice(self) -> Self::Boxed
    where
        Self: Sized;

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

crate::static_assert::object_safe!(Vector<Item = (), Target = [()]>);

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

#[cfg_attr(doc_cfg, doc(cfg(feature = "allocator-api2")))]
#[cfg(feature = "allocator-api2")]
impl<T, A> Vector for allocator_api2::vec::Vec<T, A>
where
    A: allocator_api2::alloc::Allocator,
{
    type Item = T;
    type Boxed = allocator_api2::boxed::Box<[T], A>;

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
