//! Types, traits, and functions for parsing WebAssembly [modules encoded in the binary format].
//!
//! [modules encoded in the binary format]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module

pub mod custom;
pub mod preamble;

mod binary;
mod core_indices;
mod import_sec;
mod module_section;
mod module_section_sequence;
mod type_sec;

pub use binary::Module;
pub use core_indices::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx,
};
pub use import_sec::{Import, ImportDesc, ImportSec};
pub use module_section::{ModuleSection, ModuleSectionId};
pub use module_section_sequence::{
    module_section_sequence, module_section_sequence_with_unknown, ModuleSectionOrder,
};
pub use type_sec::TypeSec;
