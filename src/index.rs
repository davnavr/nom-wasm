//! Traits for representing, and types and functions for parsing, WebAssembly [indices].
//!
//! [indices]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-index

/// A [WebAssembly index](https://webassembly.github.io/spec/core/binary/modules.html#indices).
pub trait Index:
    Copy
    + core::fmt::Debug
    + core::fmt::Display
    + From<u32>
    + PartialEq<u32>
    + PartialOrd<u32>
    + Into<u32>
    + core::hash::Hash
    + nom::ToUsize
    + Eq
    + Ord
    + Send
    + Sync
    + 'static
{
    /// A human readable string that indicates what this [`Index`] refers to.
    const NAME: &'static str;

    /// Parses a [*LEB128*](crate::values::leb128) encoded unsigned 32-bit integer [`Index`].
    #[inline]
    fn parse<'a, E: crate::error::ErrorSource<'a>>(input: &'a [u8]) -> crate::Parsed<'a, Self, E> {
        crate::values::leb128_u32(input).map(|(input, index)| (input, Self::from(index)))
    }
}

/// Provides a [`nom::Parser`] implementation for a [*LEB128*](crate::values::leb128) encoded
/// [`Index`].
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct IndexParser;

impl<'a, I, E> nom::Parser<&'a [u8], I, E> for IndexParser
where
    I: Index,
    E: crate::error::ErrorSource<'a>,
{
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, I, E> {
        <I>::parse(input)
    }
}

/// Type alias for an [`Iterator`] that parses a vector of indices.
pub type IndexVectorParser<'a, I, E> = crate::values::VectorIter<'a, I, E, IndexParser>;

/// Defines wrapper structs that represent a WebAssembly [`Index`].
///
/// The generated structs automatically derive [`Clone`], [`Copy`], [`Eq`], [`Ord`], and [`Hash`],
/// and are defined to be [`repr(transparent)`].
///
/// [`repr(transparent)`]: https://doc.rust-lang.org/nomicon/other-reprs.html#reprtransparent
macro_rules! definitions {
    {$(
        $(#[$meta:meta])*
        struct $name:ident = $desc:literal;
    )*} => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name(pub u32);

        impl $crate::nom::ToUsize for $name {
            #[inline]
            fn to_usize(&self) -> usize {
                <u32 as $crate::nom::ToUsize>::to_usize(&self.0)
            }
        }

        impl From<u32> for $name {
            #[inline]
            fn from(index: u32) -> Self {
                Self(index)
            }
        }

        impl From<$name> for u32 {
            #[inline]
            fn from(index: $name) -> u32 {
                index.0
            }
        }

        impl PartialEq<u32> for $name {
            #[inline] // trait method
            fn eq(&self, other: &u32) -> bool {
                self.0 == *other
            }
        }

        impl PartialOrd<u32> for $name {
            #[inline] // trait method
            fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
                PartialOrd::partial_cmp(&self.0, other)
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                if !f.alternate() {
                    f.debug_tuple(stringify!($name)).field(&self.0).finish()
                } else {
                    core::fmt::Debug::fmt(&self.0, f)
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl $crate::index::Index for $name {
            const NAME: &'static str = $desc;
        }
    )*};
}

pub(crate) use definitions;
