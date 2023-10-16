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

macro_rules! instr_method_define_default {
    ($name:ident($($($parameter:ident: $parameter_ty:ty),+)?)) => {
        #[inline]
        fn $name(&mut self $(, $($parameter: $parameter_ty),+)?) -> Result<(), ParseInstrError<E>> {
            $($(let _ = $parameter;)*)?
            Err(ParseInstrError::Unrecognized)
        }
    };
}

macro_rules! instr_method_declaration {
    ($macro_name:ident(br_table { targets: BrTableTargets })) => {
        $macro_name!(br_table(targets: &mut isa::BrTableTargets<'a, E>));
    };
    ($macro_name:ident(select_typed { types: SelectTypes })) => {
        $macro_name!(select_typed(types: &mut isa::SelectTypes<'a, E>));
    };
    ($macro_name:ident($name:ident $({ $($field_name:ident: $field_type:ident),+ })?)) => {
        $macro_name!($name($($($field_name: $field_type),+)?));
    };
}

macro_rules! parse_instr_method {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
    )*) => {
        $(
            instr_method_declaration!(instr_method_define_default($snake_ident $({ $($field_name: $field_type),+ })?));
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

macro_rules! instr_method_define_delegate {
    ($name:ident($($($parameter:ident: $parameter_ty:ty),+)?)) => {
        #[inline]
        fn $name(&mut self $(, $($parameter: $parameter_ty),+)?) -> Result<(), ParseInstrError<E>> {
            <P>::$name(self $(, $($parameter),+)?)
        }
    };
}

macro_rules! parse_instr_delegate_method {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
    )*) => {
        $(
            instr_method_declaration!(instr_method_define_delegate($snake_ident $({ $($field_name: $field_type),+ })?));
        )*
    };
}

impl<'a, E, P> ParseInstr<'a, E> for &mut P
where
    E: ErrorSource<'a>,
    P: ParseInstr<'a, E>,
{
    crate::isa::instr_definitions::all!(parse_instr_delegate_method);
}
