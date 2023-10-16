//! Types representing and traits and functions for parsing [WebAssembly instructions].
//!
//! [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html

mod instr_definitions;
mod instr_kind;
mod invalid_instr;
mod invalid_opcode;
mod mem_arg;
mod opcodes;
mod parse_instr;

pub use crate::module::LabelIdx;
pub use instr_kind::InstrKind;
pub use invalid_instr::InvalidInstr;
pub use invalid_opcode::InvalidOpcode;
pub use mem_arg::{Align, MemArg};
pub use opcodes::{FCPrefixedOpcode, FEPrefixedOpcode, Opcode, V128Opcode};
pub use parse_instr::{ParseInstr, ParseInstrError, Result};
