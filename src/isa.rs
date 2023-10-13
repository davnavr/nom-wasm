//! Types representing and traits and functions for parsing [WebAssembly instructions].
//!
//! [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html

mod instr_kind;
mod invalid_opcode;
mod opcodes;
mod instr_definitions;
mod parse_instr;
mod invalid_instr;

pub use instr_kind::InstrKind;
pub use invalid_opcode::InvalidOpcode;
pub use opcodes::{FCPrefixedOpcode, FEPrefixedOpcode, Opcode, V128Opcode};
pub use parse_instr::{ParseInstr, ParseInstrError, Result};
pub use invalid_instr::InvalidInstr;
