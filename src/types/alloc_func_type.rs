#![cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]

use crate::{error::ErrorSource, types::ValType};
use alloc::{boxed::Box, vec::Vec};
use nom::ToUsize;

#[cfg(feature = "allow-unsafe")]
use core::{mem::MaybeUninit, ptr::NonNull};

#[cfg(feature = "allow-unsafe")]
const INLINE_AMOUNT: usize = core::mem::size_of::<NonNull<ValType>>();

#[cfg(feature = "allow-unsafe")]
union FuncTypeStorage {
    inline: [MaybeUninit<ValType>; INLINE_AMOUNT],
    allocated: NonNull<ValType>,
}

#[cfg(not(feature = "allow-unsafe"))]
const INLINE_AMOUNT: usize = core::mem::size_of::<*const ValType>() - 1;

#[cfg(not(feature = "allow-unsafe"))]
const EMPTY_INLINE_ARRAY: [ValType; INLINE_AMOUNT] = [ValType::I32; INLINE_AMOUNT];

#[cfg(not(feature = "allow-unsafe"))]
enum FuncTypeStorage {
    Inline {
        types: [ValType; INLINE_AMOUNT],
        result_count: u8,
    },
    Allocated(Box<[ValType]>),
}

/// Provides a [`Parser`](nom::Parser) implementation for [`FuncType`]s.
#[derive(Clone)]
#[repr(transparent)]
pub struct FuncTypeParser<'a, E: ErrorSource<'a>> {
    buffer: Vec<ValType>,
    _marker: core::marker::PhantomData<dyn nom::Parser<&'a [u8], FuncType, E>>,
}

impl<'a, E: ErrorSource<'a>> From<Vec<ValType>> for FuncTypeParser<'a, E> {
    #[inline]
    fn from(buffer: Vec<ValType>) -> Self {
        Self {
            buffer,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'a, E: ErrorSource<'a>> FuncTypeParser<'a, E> {
    #[allow(missing_docs)]
    #[inline]
    pub fn new() -> Self {
        Self::from(alloc::vec::Vec::new())
    }
}

impl<'a, E: ErrorSource<'a>> Default for FuncTypeParser<'a, E> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E: ErrorSource<'a>> nom::Parser<&'a [u8], FuncType, E> for FuncTypeParser<'a, E> {
    fn parse(&mut self, input: &'a [u8]) -> nom::IResult<&'a [u8], FuncType, E> {
        let buffer = core::cell::RefCell::new(&mut self.buffer);
        let result = crate::types::func_type_with(
            || buffer.borrow_mut(),
            |mut buf, param_types| {
                debug_assert!(buf.is_empty());
                let mut param_types = crate::values::SequenceIter::from(param_types);
                buf.extend(&mut param_types);
                let _ = param_types.finish()?;
                let param_count = u32::try_from(buf.len()).unwrap_or(u32::MAX);
                Ok((buf, param_count))
            },
            |(mut buf, param_count), result_types| {
                let mut result_types = crate::values::SequenceIter::from(result_types);
                buf.extend(&mut result_types);
                let _ = result_types.finish()?;
                Ok(FuncType::from_vec(&mut buf, param_count))
            },
        )
        .parse(input);

        result
    }
}

impl<'a, E: ErrorSource<'a>> core::fmt::Debug for FuncTypeParser<'a, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncTypeParser").finish_non_exhaustive()
    }
}

/// Represents a WebAssembly [**`functype`**].
///
/// [**`functype`**]: https://webassembly.github.io/spec/core/binary/types.html#function-types
pub struct FuncType {
    storage: FuncTypeStorage,
    param_count: u32,
    #[cfg(feature = "allow-unsafe")]
    result_count: u32,
}

#[cfg(feature = "allow-unsafe")]
crate::static_assert::check_size!(FuncType, <= core::mem::size_of::<[usize; 2]>());

#[cfg(not(feature = "allow-unsafe"))]
crate::static_assert::check_size!(FuncType, <= core::mem::size_of::<[usize; 3]>());

impl FuncType {
    /// A function type with no parameters or result values.
    pub const EMPTY: Self = Self {
        #[cfg(feature = "allow-unsafe")]
        storage: FuncTypeStorage {
            inline: [MaybeUninit::uninit(); INLINE_AMOUNT],
        },
        #[cfg(not(feature = "allow-unsafe"))]
        storage: FuncTypeStorage::Inline {
            types: EMPTY_INLINE_ARRAY,
            result_count: 0,
        },
        param_count: 0,
        #[cfg(feature = "allow-unsafe")]
        result_count: 0,
    };

    /// Creates a [`FuncType`] from a vector of [`ValType`]s, then clears the vector.
    ///
    /// If `types.len() == types.capacity()`, then ownership of the underlying allocation is
    /// taken. Otherwise, a new heap allocation is made.
    ///
    /// See the documentation for `FuncType::new()` for more information.
    ///
    /// # Panics
    ///
    /// Panics if the `types.len()` exceeds [`u32::MAX`] or `parameter_count` exceeds `types.len()`.
    ///
    /// If the attempt to allocate a new heap allocation for the types on the heap fails, then
    /// [`handle_alloc_error()`] is called.
    ///
    /// [`Vec`]: alloc::vec::Vec
    /// [`handle_alloc_error()`]: alloc::alloc::handle_alloc_error
    pub fn from_vec(types: &mut alloc::vec::Vec<ValType>, parameter_count: u32) -> Self {
        let param_len = parameter_count.to_usize();

        assert!(param_len <= types.len(), "parameter count too big");

        #[cfg(feature = "allow-unsafe")]
        let result_count: u32 = types
            .len()
            .checked_sub(param_len)
            .expect("parameter count too big")
            .try_into()
            .expect("too many parameter and result types");

        if types.len() > INLINE_AMOUNT && types.len() == types.capacity() {
            Self {
                #[cfg(feature = "allow-unsafe")]
                storage: {
                    // Note that `Vec` uses `alloc::alloc`, and that it contains more than `INLINE_AMOUNT`
                    let types = core::mem::take(types).leak();
                    FuncTypeStorage {
                        allocated: NonNull::from(types).cast(),
                    }
                },
                #[cfg(not(feature = "allow-unsafe"))]
                storage: FuncTypeStorage::Allocated(core::mem::take(types).into()),
                param_count: parameter_count,
                #[cfg(feature = "allow-unsafe")]
                result_count,
            }
        } else {
            let me = Self::new(&types[..param_len], &types[param_len..]);
            types.clear();
            me
        }
    }

    /// Allocates a new [`FuncType`] with the given parameter and result types.
    ///
    /// If the total number of parameter and result types is large enough, the types may be stored
    /// in a heap allocation.
    ///
    /// # Panics
    ///
    /// Panics if the number of `parameters` or `results` exceeds [`u32::MAX`].
    ///
    /// If the attempt to allocate space for the types on the heap fails, then
    /// [`handle_alloc_error()`] is called.
    ///
    /// [`handle_alloc_error()`]: alloc::alloc::handle_alloc_error
    pub fn new(parameters: &[ValType], results: &[ValType]) -> Self {
        let param_count = parameters
            .len()
            .try_into()
            .expect("too many parameter types");

        let result_count: u32 = results.len().try_into().expect("too many result types");

        let total_count = parameters
            .len()
            .checked_add(results.len())
            .expect("too many parameter and result types");

        #[cfg(feature = "allow-unsafe")]
        let mut storage;

        #[cfg(feature = "allow-unsafe")]
        {
            let destination: &mut [MaybeUninit<ValType>];
            if total_count <= INLINE_AMOUNT {
                let inline = [MaybeUninit::uninit(); INLINE_AMOUNT];
                storage = FuncTypeStorage { inline };
                // Safety: using inline storage above
                destination = unsafe { &mut storage.inline };
            } else {
                let layout = core::alloc::Layout::array::<ValType>(total_count).unwrap();

                debug_assert_ne!(layout.size(), 0usize);

                // Safety: layout size is never 0, since `total_len > 0 && size_of::<ValType>() > 0`
                let pointer = unsafe { alloc::alloc::alloc(layout) };
                if let Some(allocation) = NonNull::new(pointer) {
                    storage = FuncTypeStorage {
                        allocated: allocation.cast(),
                    };

                    // Safety: using allocator storage above
                    destination = unsafe {
                        NonNull::slice_from_raw_parts(storage.allocated.cast(), total_count)
                            .as_mut()
                    }
                } else {
                    alloc::alloc::handle_alloc_error(layout)
                }
            }

            // Storage has been selected, now types have to be copied

            // Safety: layout of `[MaybeUninit<T>]` and `[T]` is the same
            // Safety: these ranges are in bounds
            unsafe {
                destination
                    .get_unchecked_mut(..parameters.len())
                    .copy_from_slice(core::mem::transmute::<&[ValType], _>(parameters));

                destination
                    .get_unchecked_mut(parameters.len()..)
                    .get_unchecked_mut(..results.len())
                    .copy_from_slice(core::mem::transmute::<&[ValType], _>(results));
            }
        }

        Self {
            #[cfg(feature = "allow-unsafe")]
            storage,
            #[cfg(not(feature = "allow-unsafe"))]
            storage: if total_count <= INLINE_AMOUNT {
                let mut types = EMPTY_INLINE_ARRAY;
                types[..parameters.len()].copy_from_slice(parameters);
                types[parameters.len()..][..results.len()].copy_from_slice(results);

                #[allow(clippy::cast_possible_truncation)]
                FuncTypeStorage::Inline {
                    types,
                    // Won't overflow, since `INLINE_AMOUNT < u8::MAX`
                    result_count: result_count as u8,
                }
            } else {
                let mut types = Vec::with_capacity(total_count);
                types.extend_from_slice(parameters);
                types.extend_from_slice(results);
                FuncTypeStorage::Allocated(types.into())
            },
            param_count,
            #[cfg(feature = "allow-unsafe")]
            result_count,
        }
    }

    #[cfg(feature = "allow-unsafe")]
    #[inline]
    fn types_len(&self) -> usize {
        // Note that the code in the constructors panics if the total length overflows
        self.param_count.to_usize() + self.result_count.to_usize()
    }

    #[cfg(feature = "allow-unsafe")]
    #[inline]
    fn is_inline(&self) -> bool {
        self.types_len() <= INLINE_AMOUNT
    }

    #[cfg(feature = "allow-unsafe")]
    #[inline]
    fn types(&self) -> &[ValType] {
        // Safety: `is_inline()` ensures correct storage is used
        // Safety: for inline, array contains `types_len()` valid elements
        // Safety: for allocated, pointer points to valid `[ValType; types_len()]` allocation
        unsafe {
            let len = self.types_len();
            if self.is_inline() {
                core::mem::transmute::<&[MaybeUninit<ValType>], &[ValType]>(
                    &self.storage.inline[..len],
                )
            } else {
                NonNull::slice_from_raw_parts(self.storage.allocated, len).as_ref()
            }
        }
    }

    #[cfg(not(feature = "allow-unsafe"))]
    fn types(&self) -> &[ValType] {
        match &self.storage {
            FuncTypeStorage::Allocated(types) => types,
            FuncTypeStorage::Inline {
                types,
                result_count,
            } => &types[..self.param_count.to_usize() + usize::from(*result_count)],
        }
    }

    /// Gets the parameter types.
    #[inline]
    pub fn parameters(&self) -> &[ValType] {
        #[cfg(feature = "allow-unsafe")]
        return {
            // Safety: `param_count <= types_len()`, so this is in bounds
            unsafe { self.types().get_unchecked(..self.param_count.to_usize()) }
        };

        #[cfg(not(feature = "allow-unsafe"))]
        return &self.types()[..self.param_count.to_usize()];
    }

    /// Gets the result types.
    #[inline]
    pub fn results(&self) -> &[ValType] {
        #[cfg(feature = "allow-unsafe")]
        return {
            // Safety: `param_count <= types_len()`, so this is in bounds
            unsafe { self.types().get_unchecked(self.param_count.to_usize()..) }
        };

        #[cfg(not(feature = "allow-unsafe"))]
        return &self.types()[self.param_count.to_usize()..];
    }
}

impl From<FuncType> for Box<[ValType]> {
    #[cfg(feature = "allow-unsafe")]
    fn from(func_type: FuncType) -> Self {
        if func_type.is_inline() {
            Self::from(func_type.types())
        } else {
            // Prevents a double free
            let func_type = core::mem::ManuallyDrop::new(func_type);
            let types_len = func_type.types_len();

            // Safety: `is_inline()` ensures `storage` is a heap allocation
            // Safety: pointer originates from `alloc::alloc`
            unsafe {
                Self::from_raw(core::slice::from_raw_parts_mut(
                    func_type.storage.allocated.as_ptr(),
                    types_len,
                ))
            }
        }
    }

    #[cfg(not(feature = "allow-unsafe"))]
    fn from(func_type: FuncType) -> Self {
        match func_type.storage {
            FuncTypeStorage::Inline { .. } => Self::from(func_type.types()),
            FuncTypeStorage::Allocated(types) => types,
        }
    }
}

#[cfg(feature = "allow-unsafe")]
impl Drop for FuncType {
    fn drop(&mut self) {
        // Only need to drop if a heap allocation occured
        if !self.is_inline() {
            let len = self.types_len();

            // Safety: `is_inline` ensures heap storage is being used
            let allocated = unsafe { self.storage.allocated };

            // Safety: `pointer` originates from `alloc::alloc`
            // Safety: `ValType` isn't `Drop`, so safe to just deallocate here
            unsafe {
                alloc::alloc::dealloc(
                    allocated.as_ptr() as *mut u8,
                    core::alloc::Layout::array::<ValType>(len).unwrap(),
                );
            }
        }
    }
}

impl core::fmt::Debug for FuncType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("FuncType")
            .field("parameters", &self.parameters())
            .field("results", &self.results())
            .finish()
    }
}
