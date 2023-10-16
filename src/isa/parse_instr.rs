use crate::{
    error::ErrorSource,
    isa::{self, LabelIdx, LaneIdx, MemArg},
    module::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx},
    types::{BlockType, RefType},
    values::{V128ShuffleLanes, F32, F64, V128},
};

/// Error type used by the [`ParseInstr`] trait's methods.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum ParseInstrError<E> {
    /// An immediate argument for the parsed instruction could not be parsed.
    ParseFailed(E),
    /// The [`ParseInstr`] trait does not recognize the instruction that was parsed.
    Unrecognized,
}

impl<E: core::fmt::Display> core::fmt::Display for ParseInstrError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParseFailed(err) => core::fmt::Display::fmt(err, f),
            Self::Unrecognized => f.write_str("instruction was not recognized"),
        }
    }
}

/// Result type used by the [`ParseInstr`] trait's methods.
pub type Result<T, E> = core::result::Result<T, ParseInstrError<E>>;

macro_rules! parse_instr_method_definition {
    (br_table { targets: BrTableTargets }) => {
        #[inline]
        fn br_table(&mut self, targets: &mut isa::BrTableTargets<'a, E>) -> Result<(), ParseInstrError<E>> {
            let _ = targets;
            Err(ParseInstrError::Unrecognized)
        }
    };
    (select_typed { types: SelectTypes }) => {
        #[inline]
        fn select_typed(&mut self, types: &mut isa::SelectTypes<'a, E>) -> Result<(), ParseInstrError<E>> {
            let _ = types;
            Err(ParseInstrError::Unrecognized)
        }
    };
    ($snake_ident:ident $({ $($field_name:ident: $field_type:ident),+ })?) => {
        #[inline]
        fn $snake_ident(&mut self $(, $($field_name: $field_type),+ )?) -> Result<(), ParseInstrError<E>> {
            $(
                $(let _ = $field_name;)+
            )?
            Err(ParseInstrError::Unrecognized)
        }
    };
}

macro_rules! parse_instr_method {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
    )*) => {
        $(
            parse_instr_method_definition!($snake_ident $({ $($field_name: $field_type),+ })?);
        )*
    };
}

/// Trait for parsing [WebAssembly instructions].
///
/// [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html
#[allow(missing_docs)]
pub trait ParseInstr<'a, E: ErrorSource<'a>> {
    crate::isa::instr_definitions::all!(parse_instr_method);
}

impl<'a, E: ErrorSource<'a>> ParseInstr<'a, E> for () {}
