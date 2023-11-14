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

#[cfg(feature = "allocator-api2")]
pub use allocator_api2;

mod hex;
mod parser;
mod static_assert;
mod tag;

pub mod error;
pub mod index;
pub mod input;
pub mod isa;
pub mod module;
pub mod ordering;
pub mod section;
pub mod types;
pub mod values;

/// Type alias for the result of parsing functions in [`nom-wasm`](crate).
pub type Parsed<'a, T, E = error::Error<'a>> = nom::IResult<&'a [u8], T, E>;
