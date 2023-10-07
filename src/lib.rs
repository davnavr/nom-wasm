//! A [`nom`] library for parsing the [WebAssembly binary format], with a focus on zero allocations.
//!
//! [WebAssembly binary format]: https://webassembly.github.io/spec/core/binary/index.html

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::std_instead_of_alloc)]
#![deny(clippy::exhaustive_enums)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use nom;

pub mod custom;
pub mod error;
pub mod input;
pub mod leb128;
pub mod module;
pub mod name;
pub mod ordering;
pub mod section;
pub mod sequence;

/// Type alias for the result of parsing functions in [`nom-wasm`](crate).
pub type Parsed<'a, T, E = error::Error<'a>> = nom::IResult<&'a [u8], T, E>;
