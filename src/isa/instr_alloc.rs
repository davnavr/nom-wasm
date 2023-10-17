use crate::{
    isa::{LabelIdx, LaneIdx, MemArg},
    module::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx},
    storage::Heap,
    types::{BlockType, RefType},
    values::{V128ShuffleLanes, F32, F64, V128},
};

macro_rules! instr_enum_cases {
    (@start $($tokens:tt)*) => {
        instr_enum_cases! {
            <H, f> {} {} $($tokens)*
        }
    };
    (
        <$heap:ident, $fmt:ident>
        {$($enum_cases:tt)*}
        {$($debug_cases:tt)*}
        BrTable { targets: BrTableTargets };
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            <$heap, $fmt>
            {
                $($enum_cases)*
                #[allow(missing_docs)]
                BrTable { targets: <$heap as Heap>::Box<[LabelIdx]>, default_target: LabelIdx },
            }
            {
                $($debug_cases)*
                Self::BrTable { targets, default_target } => {
                    let targets: &[LabelIdx] = targets;
                    $fmt.debug_struct("BrTable")
                        .field("targets", &targets)
                        .field("default_target", default_target)
                        .finish()
                }
            }
            $($remaining)*
        }
    };
    (
        <$heap:ident, $fmt:ident>
        {$($enum_cases:tt)*}
        {$($debug_cases:tt)*}
        Select;
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            <$heap, $fmt>
            {
                $($enum_cases)*
                #[allow(missing_docs)]
                Select { types: <$heap as Heap>::Box<[crate::types::ValType]> },
            }
            {
                $($debug_cases)*
                Self::Select { types } => {
                    let mut s = $fmt.debug_tuple("Select");
                    let types: &[crate::types::ValType] = types;
                    if !types.is_empty() {
                        s.field(&types);
                    }
                    s.finish()
                }
            }
            $($remaining)*
        }
    };
    (
        <$heap:ident, $fmt:ident>
        {$($enum_cases:tt)*}
        {$($debug_cases:tt)*} SelectTyped { types: SelectTypes };
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            <$heap, $fmt>
            {
                $($enum_cases)*
                // This case is already handled by Select
            }
            {
                $($debug_cases)*
                // This case is already handled by Select
            }
            $($remaining)*
        }
    };
    (
        <$heap:ident, $fmt:ident>
        {$($enum_cases:tt)*}
        {$($debug_cases:tt)*}
        $pascal_ident:ident $({ $field_name:ident: $field_type:ident })?;
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            <$heap, $fmt>
            {
                $($enum_cases)*
                #[allow(missing_docs)]
                $pascal_ident $(($field_type))?,
            }
            {
                $($debug_cases)*
                Self::$pascal_ident $(($field_name))? => {
                    let mut t = $fmt.debug_tuple(stringify!($pascal_ident));
                    $(let t = t.field($field_name);)?
                    t.finish()
                }
            }
            $($remaining)*
        }
    };
    (
        <$heap:ident, $fmt:ident>
        {$($enum_cases:tt)*} {$($debug_cases:tt)*}
        $pascal_ident:ident { $($field_name:ident: $field_type:ident),+ };
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            <$heap, $fmt>
            {
                $($enum_cases)*
                #[allow(missing_docs)]
                $pascal_ident { $($field_name: $field_type),+ },
            }
            {
                $($debug_cases)*
                Self::$pascal_ident { $($field_name),+ } => {
                    let mut s = $fmt.debug_struct(stringify!($pascal_ident));
                    $(let s = s.field(stringify!($field_name), $field_name);)+
                    s.finish()
                }
            }
            $($remaining)*
        }
    };
    (<$heap:ident, $fmt:ident> {$($enum_cases:tt)*} {$($debug_cases:tt)*}) => {
        /// Represents a WebAssembly [instruction].
        ///
        /// [instruction]: https://webassembly.github.io/spec/core/binary/instructions.html
        #[non_exhaustive]
        pub enum Instr<$heap: Heap> {
            $($enum_cases)*
            #[doc(hidden)]
            _MarkerForUnusedGeneric(core::marker::PhantomData<fn() -> H>),
        }

        impl<H: Heap> core::fmt::Debug for Instr<H> {
            fn fmt(&self, $fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self {
                    $($debug_cases)*
                    Self::_MarkerForUnusedGeneric { .. } => Ok(()),
                }
            }
        }
    };
}

macro_rules! define_instr_enum {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
    )*) => {
        instr_enum_cases!(@start $($pascal_ident $({ $($field_name: $field_type),+ })?;)*);
        //impl Clone, Copy and Debug manually
    };
}

crate::isa::instr_definitions::all!(define_instr_enum);
