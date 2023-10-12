//! Types representing and traits and functions for parsing [WebAssembly instructions].
//!
//! [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html

mod instr_kind;
mod invalid_opcode;
mod opcodes;

pub use instr_kind::InstrKind;
pub use invalid_opcode::InvalidOpcode;
pub use opcodes::{FCPrefixedOpcode, FEPrefixedOpcode, Opcode, V128Opcode};
