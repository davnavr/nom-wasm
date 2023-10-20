//! Contains types representing [WebAssembly instructions].
//!
//! This module is dependent on the `allocator-api2` feature.
//!
//! [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html

use crate::{
    error::ErrorSource,
    isa,
    module::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx},
    types::{BlockType, RefType, ValType},
    values::{V128ShuffleLanes, F32, F64, V128},
};
use allocator_api2::{
    alloc::{Allocator, Global},
    boxed::Box,
    vec::Vec,
};
use core::{
    fmt::{Debug, Formatter},
    hash::Hash,
    marker::PhantomData,
};

pub use isa::{InstrKind, LabelIdx, LaneIdx, MemArg};

macro_rules! instr_kind_case {
    (Byte => $name:ident) => {
        InstrKind::Byte(isa::Opcode::$name)
    };
    (FCPrefixed => $name:ident) => {
        InstrKind::FCPrefixed(isa::FCPrefixedOpcode::$name)
    };
    (V128 => $name:ident) => {
        InstrKind::V128(isa::V128Opcode::$name)
    };
    (FEPrefixed => $name:ident) => {
        InstrKind::FEPrefixed(isa::FEPrefixedOpcode::$name)
    };
}

macro_rules! instr_case_common {
    ($opcode_case:ident $wasm_name:literal $pascal_ident:ident) => {
        #[allow(missing_docs)]
        impl<A: Allocator> $pascal_ident<A> {
            pub const NAME: &'static str = $wasm_name;
            pub const OPCODE: InstrKind = instr_kind_case!($opcode_case => $pascal_ident);
        }
    };
}

macro_rules! instr_case_common_debug {
    ($pascal_ident:ident $({ $field_name:ident: $_field_type:ident })?) => {
        impl<A: Allocator> Debug for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.debug_tuple(stringify!($pascal_ident))
                    $(.field(&self.$field_name))?
                    .finish()
            }
        }
    };
    ($pascal_ident:ident {$($field_name:ident: $_field_type:ident),+}) => {
        impl<A: Allocator> Debug for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.debug_struct(stringify!($pascal_ident))
                    $(.field(stringify!($field_name), &self.$field_name))+
                    .finish()
            }
        }
    };
}

macro_rules! instr_case_common_partial_eq {
    ($pascal_ident:ident) => {
        impl<A1: Allocator, A2: Allocator> PartialEq<$pascal_ident<A2>> for $pascal_ident<A1> {
            #[inline]
            fn eq(&self, other: &$pascal_ident<A2>) -> bool {
                let _ = other;
                true
            }
        }
    };
    ($pascal_ident:ident { $field_name:ident: $_field_type:ident }) => {
        impl<A1: Allocator, A2: Allocator> PartialEq<$pascal_ident<A2>> for $pascal_ident<A1> {
            #[inline]
            fn eq(&self, other: &$pascal_ident<A2>) -> bool {
                self.$field_name == other.$field_name
            }
        }
    };
    ($pascal_ident:ident {$($field_name:ident: $_field_type:ident),+}) => {
        impl<A1: Allocator, A2: Allocator> PartialEq<$pascal_ident<A2>> for $pascal_ident<A1> {
            #[inline]
            fn eq(&self, other: &$pascal_ident<A2>) -> bool {
                $(self.$field_name == other.$field_name)&&+
            }
        }
    };
}

macro_rules! instr_case {
    (Byte $wasm_name:literal BrTable { targets: BrTableTargets }) => {
        #[derive(Clone)]
        #[allow(missing_docs)]
        #[non_exhaustive]
        pub struct BrTable<A: Allocator = Global> {
            pub targets: Box<[LabelIdx], A>,
            pub default_target: LabelIdx,
        }

        instr_case_common!(Byte $wasm_name BrTable);

        impl<A1: Allocator, A2: Allocator> PartialEq<BrTable<A2>> for BrTable<A1> {
            #[inline]
            fn eq(&self, other: &BrTable<A2>) -> bool {
                let self_targets: &[LabelIdx] = &*self.targets;
                let other_targets: &[LabelIdx] = &*other.targets;
                self.default_target == other.default_target && self_targets == other_targets
            }
        }

        impl<A: Allocator> Eq for BrTable<A> {}

        impl<A: Allocator> Hash for BrTable<A> {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                <&[LabelIdx]>::hash(&&*self.targets, state);
                self.default_target.hash(state);
            }
        }

        impl<A: Allocator> Debug for BrTable<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.debug_struct("BrTable")
                    .field("targets", &&*self.targets)
                    .field("default_target", &self.default_target)
                    .finish()
            }
        }
    };
    (Byte $wasm_name:literal SelectTyped { types: SelectTypes }) => {
        #[derive(Clone)]
        #[allow(missing_docs)]
        #[non_exhaustive]
        pub struct SelectTyped<A: Allocator = Global> {
            operand_type: ValType,
            _marker: PhantomData<fn() -> A>
        }

        instr_case_common!(Byte $wasm_name SelectTyped);

        impl<A: Allocator> SelectTyped<A> {
            /// Returns the [`ValType`] of the operand to the [`select` instruction].
            ///
            /// # Errors
            ///
            /// Returns an error if more than one [`ValType`] is specified, a case that is
            /// currently not supported by [`nom_wasm`](crate).
            ///
            /// [`select` instruction]: https://webassembly.github.io/spec/core/binary/instructions.html#control-instructions
            pub fn to_val_type(self) -> Result<ValType, Self> {
                // TODO: Figure out if a Option<ValType> should be used, does `select` allow empty vec of types?
                Ok(self.operand_type)
            }
        }

        impl<A1: Allocator, A2: Allocator> PartialEq<SelectTyped<A2>> for SelectTyped<A1> {
            #[inline]
            fn eq(&self, other: &SelectTyped<A2>) -> bool {
                self.operand_type == other.operand_type
            }
        }

        impl<A: Allocator> Eq for SelectTyped<A> {}

        impl<A: Allocator> Hash for SelectTyped<A> {
            #[inline]
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                <&[ValType]>::hash(&[self.operand_type].as_slice(), state)
            }
        }

        impl<A: Allocator> Debug for SelectTyped<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.debug_list().entry(&self.operand_type).finish()
            }
        }
    };
    {
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({
            $($field_name:ident: $field_type:ident),+
        })?
    } => {
        #[derive(Clone, Copy)]
        #[allow(missing_docs)]
        #[non_exhaustive]
        pub struct $pascal_ident<A: Allocator = Global> {
            $($(
                pub $field_name: $field_type,
            )+)?
            _marker: PhantomData<fn() -> A>
        }

        instr_case_common!($opcode_case $wasm_name $pascal_ident);

        instr_case_common_partial_eq!($pascal_ident $({$($field_name: $field_type),+})?);

        impl<A: Allocator> Eq for $pascal_ident<A> {}

        impl<A: Allocator> Hash for $pascal_ident<A> {
            #[inline]
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                let _ = state;
                $($(self.$field_name.hash(state);)+)?
            }
        }

        instr_case_common_debug!($pascal_ident $({$($field_name: $field_type),+})?);
    };
}

/// Error type used in [`ParseExpr`] to indicate that an [`Instr`]uction is not recognized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnrecognizedInstr;

impl core::fmt::Display for UnrecognizedInstr {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str("instruction was not recognzied")
    }
}

/// Trait for parsing [`Instr`]uctions in expressions.
pub trait ParseExpr<A: Allocator = Global> {
    #[allow(missing_docs)]
    fn parse(&mut self, instr: Instr<A>) -> Result<(), UnrecognizedInstr>;
}

crate::static_assert::object_safe!(ParseExpr);

impl<A: Allocator> ParseExpr<A> for Vec<Instr<A>, A> {
    #[inline]
    fn parse(&mut self, instr: Instr<A>) -> Result<(), UnrecognizedInstr> {
        self.push(instr);
        Ok(())
    }
}

/// Provides a [`ParseInstr`](isa::ParseInstr) implementation for a [`ParseExpr`] implementation.
pub struct Parser<'a, E, P, A = Global>
where
    E: ErrorSource<'a>,
    P: ParseExpr<A>,
    A: Clone + Allocator,
{
    allocator: A,
    parser: P,
    _marker: PhantomData<fn(&'a [u8]) -> E>,
}

#[allow(missing_docs)]
impl<'a, E, P, A> Parser<'a, E, P, A>
where
    E: ErrorSource<'a>,
    P: ParseExpr<A>,
    A: Clone + Allocator,
{
    pub fn with_allocator(parser: P, allocator: A) -> Self {
        Self {
            allocator,
            parser,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn allocator(&self) -> &A {
        &self.allocator
    }

    #[inline]
    pub fn into_parser(self) -> P {
        self.parser
    }

    fn br_table_impl(&mut self, targets: &mut isa::BrTableTargets<'a, E>) -> isa::Result<(), E> {
        let mut other_targets = Vec::with_capacity_in(targets.len() - 1, self.allocator.clone());
        let mut default_target = LabelIdx(0);
        while let Some(result) = targets.next() {
            let label = result?;
            if targets.len() == 0 {
                default_target = label;
            } else {
                other_targets.push(label);
            }
        }

        let instr = Instr::BrTable(BrTable {
            targets: other_targets.into_boxed_slice(),
            default_target,
        });

        self.parser
            .parse(instr)
            .map_err(|UnrecognizedInstr| isa::ParseInstrError::Unrecognized)
    }

    fn select_typed_impl(&mut self, types: &mut isa::SelectTypes<'a, E>) -> isa::Result<(), E> {
        let start = crate::input::AsInput::as_input(types);
        let result = types
            .next()
            .expect("SelectTypes always returns at least 1 type");
        let operand_type = result?;

        if types.len() > 0 {
            let arity = u8::try_from(types.len())
                .ok()
                .and_then(|a| a.checked_add(1))
                .and_then(core::num::NonZeroU8::new)
                .unwrap_or(core::num::NonZeroU8::MAX);

            let e = E::from_error_kind_and_cause(
                start,
                crate::error::ErrorKind::Verify,
                crate::error::ErrorCause::Instr {
                    opcode: InstrKind::Byte(isa::Opcode::SelectTyped),
                    reason: isa::InvalidInstr::SelectTypedArity(arity),
                },
            );

            return Err(isa::ParseInstrError::Nom(nom::Err::Failure(e)));
        }

        let instr = Instr::SelectTyped(SelectTyped {
            operand_type,
            _marker: PhantomData,
        });

        self.parser
            .parse(instr)
            .map_err(|UnrecognizedInstr| isa::ParseInstrError::Unrecognized)
    }
}

impl<'a, E, P, A> Debug for Parser<'a, E, P, A>
where
    E: ErrorSource<'a>,
    P: ParseExpr<A>,
    A: Clone + Allocator,
{
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.debug_struct("Parser").finish_non_exhaustive()
    }
}

macro_rules! parse_method_impl {
    (br_table<$_lifetime:lifetime, $error:ident>(targets: BrTableTargets) => BrTable) => {
        #[inline]
        fn br_table(&mut self, targets: &mut isa::BrTableTargets<'a, E>) -> isa::Result<(), $error> {
            self.br_table_impl(targets)
        }
    };
    (select_typed<$_lifetime:lifetime, $error:ident>(types: SelectTypes) => SelectTyped) => {
        #[inline]
        fn select_typed(&mut self, types: &mut isa::SelectTypes<'a, E>) -> isa::Result<(), $error> {
            self.select_typed_impl(types)
        }
    };
    ($snake_ident:ident<$_lifetime:lifetime, $error:ident>($($($field_name:ident: $field_type:ident),+)?) => $pascal_ident:ident) => {
        fn $snake_ident(&mut self $(, $($field_name: $field_type),+)?) -> isa::Result<(), $error> {
            let instr = Instr::$pascal_ident($pascal_ident {
                _marker: PhantomData
                $(, $($field_name),+)?
            });

            self.parser
                .parse(instr)
                .map_err(|UnrecognizedInstr| isa::ParseInstrError::Unrecognized)
        }
    };
}

macro_rules! instr_enum {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
    )*) => {
        /// Represents a WebAssembly [instruction].
        ///
        /// [instruction]: https://webassembly.github.io/spec/core/binary/instructions.html
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum Instr<A: Allocator = Global> {
            $(
                #[allow(missing_docs)]
                $pascal_ident($pascal_ident<A>),
            )*
        }

        $(
            instr_case! {
                $opcode_case $wasm_name $pascal_ident $({
                    $($field_name: $field_type),+
                })?
            }
        )*

        #[allow(missing_docs)]
        impl<A: Allocator> Instr<A> {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$pascal_ident(_) => $wasm_name,)*
                }
            }

            pub fn opcode(&self) -> InstrKind {
                match self {
                    $(Self::$pascal_ident(_) => instr_kind_case!($opcode_case => $pascal_ident),)*
                }
            }
        }

        impl<A1: Allocator, A2: Allocator> PartialEq<Instr<A2>> for Instr<A1> {
            fn eq(&self, other: &Instr<A2>) -> bool {
                match (self, other) {
                    $((Self::$pascal_ident(x), Instr::$pascal_ident(y)) => x == y,)*
                    _ => false,
                }
            }
        }

        impl<A: Allocator> Eq for Instr<A> {}

        impl<A: Allocator> Hash for Instr<A> {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.opcode().hash(state);
                match self {
                    $(Self::$pascal_ident(instr) => instr.hash(state),)*
                }
            }
        }

        impl<A: Allocator> Debug for Instr<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                match self {
                    $(Self::$pascal_ident(ident) => Debug::fmt(ident, f),)*
                }
            }
        }

        impl<'a, E, P, A> isa::ParseInstr<'a, E> for Parser<'a, E, P, A>
        where
            E: ErrorSource<'a>,
            P: ParseExpr<A>,
            A: Clone + Allocator,
        {
            $(parse_method_impl!($snake_ident<'a, E> ($($($field_name: $field_type),+)?) => $pascal_ident);)*
        }
    };
}

crate::isa::instr_definitions::all!(instr_enum);
