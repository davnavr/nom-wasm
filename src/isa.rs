//! Types representing and traits and functions for parsing [WebAssembly instructions].
//!
//! [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html

mod br_table_targets;
mod expr;
mod instr_definitions;
mod invalid_instr;
mod invalid_opcode;
mod mem_arg;
mod opcode;
mod opcode_enums;
mod parse_instr;
mod parse_instruction;

#[cfg_attr(doc_cfg, doc(cfg(feature = "allocator-api2")))]
#[cfg(feature = "allocator-api2")]
pub mod instructions;

pub use crate::module::LabelIdx;
pub use br_table_targets::BrTableTargets;
pub use expr::{expr, InvalidExpr};
pub use invalid_instr::InvalidInstr;
pub use invalid_opcode::InvalidOpcode;
pub use mem_arg::{Align, MemArg};
pub use opcode::Opcode;
pub use opcode_enums::{ByteOpcode, FCPrefixedOpcode, FEPrefixedOpcode, V128Opcode};
pub use parse_instr::{ParseInstr, ParseInstrError, Result};
pub use parse_instruction::instr;

/// A WebAssembly [**`laneidx`**] refers to a lane within a 128-bit vector.
///
/// [**`laneidx`**]: https://webassembly.github.io/spec/core/binary/instructions.html#vector-instructions
pub type LaneIdx = u8;

/// Parses the types of a [typed `select`] instruction.
///
/// [typed `select`]: ParseInstr::select_typed
pub type SelectTypes<'a, E> =
    crate::values::BoundedVectorIter<'a, 1, crate::types::ValType, E, crate::types::ValTypeParser>;
