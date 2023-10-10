//! Types and functions for parsing WebAssembly [modules encoded in the binary format].
//!
//! [modules encoded in the binary format]: https://webassembly.github.io/spec/core/binary/modules.html#binary-module

pub mod custom;
pub mod preamble;

mod core_indices;
mod module_section;
//mod module_section_sequence;
mod import_sec;
mod type_sec;

pub use core_indices::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, MemIdx, TableIdx, TagIdx, TypeIdx,
};
pub use module_section::{ModuleSection, ModuleSectionId};
//pub use module_section_sequence::{ModuleSectionOrder, ModuleSectionSequence};
pub use import_sec::{Import, ImportDesc, ImportSec};
pub use type_sec::TypeSec;
