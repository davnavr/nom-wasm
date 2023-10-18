//! Contains types representing [WebAssembly instructions].
//!
//! This module is dependent on the `allocator-api2` feature.
//!
//! [WebAssembly instructions]: https://webassembly.github.io/spec/core/binary/instructions.html

use crate::{
    isa,
    module::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx},
    types::{BlockType, RefType, ValType},
    values::{V128ShuffleLanes, F32, F64, V128},
};
use allocator_api2::{
    alloc::{Allocator, Global},
    boxed::Box,
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
    };
}

crate::isa::instr_definitions::all!(instr_enum);
