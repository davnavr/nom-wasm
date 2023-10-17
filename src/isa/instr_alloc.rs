use crate::{
    isa::{LabelIdx, LaneIdx, MemArg},
    module::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LocalIdx, MemIdx, TableIdx, TagIdx, TypeIdx},
    storage::Heap,
    types::{BlockType, RefType, ValType},
    values::{V128ShuffleLanes, F32, F64, V128},
};

macro_rules! instr_enum_cases {
    (@start $($tokens:tt)*) => {
        instr_enum_cases! {
            enum<H> {}

            Debug(f) {}

            Clone {}

            $($tokens)*
        }
    };
    (
        enum<$heap:ident> {$($enum_cases:tt)*}
        Debug($fmt:ident) {$($debug_cases:tt)*}
        Clone {$($clone_cases:tt)*}
        BrTable { targets: BrTableTargets };
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            enum<$heap> {
                $($enum_cases)*
                #[allow(missing_docs)]
                BrTable { targets: <$heap as Heap>::Box<[LabelIdx]>, default_target: LabelIdx },
            }

            Debug($fmt) {
                $($debug_cases)*
                Self::BrTable { targets, default_target } => {
                    let targets: &[LabelIdx] = targets;
                    $fmt.debug_struct("BrTable")
                        .field("targets", &targets)
                        .field("default_target", default_target)
                        .finish()
                }
            }

            Clone {
                $($clone_cases)*
                Self::BrTable { targets, default_target } => Self::BrTable {
                    targets: targets.clone(),
                    default_target: *default_target,
                },
            }

            $($remaining)*
        }
    };
    (
        enum<$heap:ident> {$($enum_cases:tt)*}
        Debug($fmt:ident) {$($debug_cases:tt)*}
        Clone {$($clone_cases:tt)*}
        Select;
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            enum<$heap> {
                $($enum_cases)*
                #[allow(missing_docs)]
                Select { types: <$heap as Heap>::Box<[ValType]> },
            }

            Debug($fmt) {
                $($debug_cases)*
                Self::Select { types } => {
                    let mut s = $fmt.debug_tuple("Select");
                    let types: &[ValType] = types;
                    if !types.is_empty() {
                        s.field(&types);
                    }
                    s.finish()
                }
            }

            Clone {
                $($clone_cases)*
                Self::Select { types } => Self::Select { types: types.clone() },
            }

            $($remaining)*
        }
    };
    (
        enum<$heap:ident> {$($enum_cases:tt)*}
        Debug($fmt:ident) {$($debug_cases:tt)*}
        Clone {$($clone_cases:tt)*}
        SelectTyped { types: SelectTypes };
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            enum<$heap> {
                $($enum_cases)*
                // This case is already handled by Select
            }

            Debug($fmt) {
                $($debug_cases)*
                // This case is already handled by Select
            }

            Clone {
                $($clone_cases)*
                // This case is already handled by Select
            }

            $($remaining)*
        }
    };
    (
        enum<$heap:ident> {$($enum_cases:tt)*}
        Debug($fmt:ident) {$($debug_cases:tt)*}
        Clone {$($clone_cases:tt)*}
        $pascal_ident:ident $({ $field_name:ident: $field_type:ident })?;
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            enum<$heap> {
                $($enum_cases)*
                #[allow(missing_docs)]
                $pascal_ident $(($field_type))?,
            }

            Debug($fmt) {
                $($debug_cases)*
                Self::$pascal_ident $(($field_name))? => {
                    let mut t = $fmt.debug_tuple(stringify!($pascal_ident));
                    $(let t = t.field($field_name);)?
                    t.finish()
                }
            }

            Clone {
                $($clone_cases)*
                Self::$pascal_ident $(($field_name))? => {
                    Self::$pascal_ident $(($field_name.clone()))?
                }
            }

            $($remaining)*
        }
    };
    (
        enum<$heap:ident> {$($enum_cases:tt)*}
        Debug($fmt:ident) {$($debug_cases:tt)*}
        Clone {$($clone_cases:tt)*}
        $pascal_ident:ident { $($field_name:ident: $field_type:ident),+ };
        $($remaining:tt)*
    ) => {
        instr_enum_cases! {
            enum<$heap> {
                $($enum_cases)*
                #[allow(missing_docs)]
                $pascal_ident { $($field_name: $field_type),+ },
            }

            Debug($fmt) {
                $($debug_cases)*
                Self::$pascal_ident { $($field_name),+ } => {
                    let mut s = $fmt.debug_struct(stringify!($pascal_ident));
                    $(let s = s.field(stringify!($field_name), $field_name);)+
                    s.finish()
                }
            }

            Clone {
                $($clone_cases)*
                Self::$pascal_ident { $($field_name),+ } => Self::$pascal_ident {
                    $($field_name: $field_name.clone()),+
                },
            }

            $($remaining)*
        }
    };
    {
        enum<$heap:ident> {
            $($enum_cases:tt)*
        }

        Debug($fmt:ident) {
            $($debug_cases:tt)*
        }

        Clone {
            $($clone_cases:tt)*
        }
    } => {
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

        impl<H> Clone for Instr<H>
        where
            H: Heap,
            <H as Heap>::Box<[ValType]>: Clone,
            <H as Heap>::Box<[LabelIdx]>: Clone,
        {
            fn clone(&self) -> Self {
                match self {
                    $($clone_cases)*
                    Self::_MarkerForUnusedGeneric { .. } => Self::_MarkerForUnusedGeneric(core::marker::PhantomData),
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
