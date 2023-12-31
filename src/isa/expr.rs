use crate::{
    error::{ErrorCause, ErrorSource},
    isa::{self, LabelIdx, LaneIdx, MemArg, ParseInstr, Result},
    module::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx},
    types::{BlockType, RefType},
    values::{V128ShuffleLanes, F32, F64, V128},
};

/// Describes an error that occured while parsing a WebAssembly [**`expr`**](expr).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum InvalidExpr {
    ExpectedEnds(u32),
    BlockNestingOverflow,
}

impl core::fmt::Display for InvalidExpr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::ExpectedEnds(count) => {
                write!(f, "expected {count} more `end` instructions in expression")
            }
            Self::BlockNestingOverflow => {
                f.write_str("block nesting counter overflowed while parsing expression")
            }
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidExpr {}

struct ParseExprInstr<'a, P, E>
where
    P: ParseInstr<'a, E>,
    E: ErrorSource<'a>,
{
    block_nesting: u32,
    parser: P,
    _marker: core::marker::PhantomData<fn(&'a [u8]) -> E>,
}

macro_rules! update_block_count {
    ($self:ident @ block | r#loop | r#if | r#try) => {
        if let Some(level) = self.block_nesting.checked_add(1) {
            $self.block_nesting = level;
        } else {
            return Err(isa::ParseInstrError::Cause(ErrorCause::Expr(
                InvalidExpr::BlockNestingOverflow,
            )));
        }
    };
    ($self:ident @ end) => {
        $self.block_nesting -= 1;
    };
    ($self:ident @ delegate) => {
        if $self.block_nesting > 1 {
            // Check above ensures a `delegate` won't erroneously mark the end of an expression
            $self.block_nesting -= 1;
        } else {
            return Err(isa::ParseInstrError::Cause(ErrorCause::Expr(
                InvalidExpr::ExpectedEnds(1),
            )));
        }
    };
    ($self:ident @ $unused: ident) => {};
}

macro_rules! parse_expr_method {
    ($name:ident($($($parameter:ident: $parameter_ty:ty),+)?)) => {
        #[inline]
        fn $name(&mut self $(, $($parameter: $parameter_ty),+)?) -> Result<(), E> {
            update_block_count!(self @ $name);
            self.parser.$name($($($parameter),+)?)
        }
    };
}

macro_rules! parse_expr_definitions {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
    )*) => {
        $(
            isa::parse_instr::instr_method_declaration!(parse_expr_method($snake_ident $({ $($field_name: $field_type),+ })?));
        )*
    };
}

impl<'a, P, E> ParseInstr<'a, E> for ParseExprInstr<'a, P, E>
where
    P: ParseInstr<'a, E>,
    E: ErrorSource<'a>,
{
    crate::isa::instr_definitions::all!(parse_expr_definitions);
}

/// Parses a [WebAssembly expression], which is a sequence of instructions terminated with an
/// [**`end`**] instruction.
///
/// [WebAssembly expression]: https://webassembly.github.io/spec/core/binary/instructions.html#expressions
/// [**`end`**]: ParseInstr::end
pub fn expr<'a, P, E>(mut input: &'a [u8], parser: P) -> crate::Parsed<'a, P, E>
where
    P: ParseInstr<'a, E>,
    E: ErrorSource<'a>,
{
    let mut state = ParseExprInstr {
        block_nesting: 1, // WASM expressions start with an implicit `block`
        parser,
        _marker: core::marker::PhantomData,
    };

    while state.block_nesting > 0 {
        input = isa::instr(input, &mut state)?.0;
    }

    Ok((input, state.parser))
}
