//! Types and functions for parsing WebAssembly [modules encodded in the binary format].
//!
//! [modules encodded in the binary format]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module

pub mod preamble;

mod module_section;

pub use module_section::{ModuleSection, ModuleSectionId};
