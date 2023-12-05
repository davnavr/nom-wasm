//! Contains types representing [WebAssembly instructions].
//!
//! The [`Instr`] enumeration represents an instruction, while the [`Parser`] struct and
//! [`ParseExpr`] trait are used for parsing [`Instr`]uctions.
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
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    marker::PhantomData,
};

pub use isa::{LabelIdx, LaneIdx, MemArg, Opcode};

macro_rules! instr_case_common {
    ($opcode_enum:ident $wasm_name:literal $pascal_ident:ident) => {
        #[allow(missing_docs)]
        impl<A: Allocator> $pascal_ident<A> {
            pub const NAME: &'static str = $wasm_name;
            pub const OPCODE: Opcode = Opcode::$pascal_ident;
            pub const OPCODE_CATEGORY: isa::$opcode_enum = isa::$opcode_enum::$pascal_ident;
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
    ($pascal_ident:ident { $($field_name:ident: $_field_type:ident),+ }) => {
        impl<A: Allocator> Debug for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.debug_struct(stringify!($pascal_ident))
                    $(.field(stringify!($field_name), &self.$field_name))+
                    .finish()
            }
        }
    };
}

const fn mem_arg_natural_align(name: &'static str) -> isa::Align {
    match name.as_bytes() {
        b"i32.load8_s"
        | b"i32.load8_u"
        | b"i64.load8_s"
        | b"i64.load8_u"
        | b"i32.store8"
        | b"i64.store8"
        | b"v128.load8_splat"
        | b"v128.load8_lane"
        | b"v128.store8_lane"
        | b"i32.atomic.load8_u"
        | b"i64.atomic.load8_u"
        | b"i32.atomic.store8_u"
        | b"i64.atomic.store8_u"
        | b"i32.atomic.rmw8.add_u"
        | b"i64.atomic.rmw8.add_u"
        | b"i32.atomic.rmw8.sub_u"
        | b"i64.atomic.rmw8.sub_u"
        | b"i32.atomic.rmw8.and_u"
        | b"i64.atomic.rmw8.and_u"
        | b"i32.atomic.rmw8.or_u"
        | b"i64.atomic.rmw8.or_u"
        | b"i32.atomic.rmw8.xor_u"
        | b"i64.atomic.rmw8.xor_u"
        | b"i32.atomic.rmw8.xchg_u"
        | b"i64.atomic.rmw8.xchg_u"
        | b"i32.atomic.rmw8.cmpxchg_u"
        | b"i64.atomic.rmw8.cmpxchg_u" => isa::Align::Any,
        b"i32.load16_s"
        | b"i32.load16_u"
        | b"i32.store16"
        | b"i64.store16"
        | b"i64.load16_s"
        | b"i64.load16_u"
        | b"v128.load16_splat"
        | b"v128.load16_lane"
        | b"v128.store16_lane"
        | b"i32.atomic.load16_u"
        | b"i64.atomic.load16_u"
        | b"i32.atomic.store16_u"
        | b"i64.atomic.store16_u"
        | b"i32.atomic.rmw16.add_u"
        | b"i64.atomic.rmw16.add_u"
        | b"i32.atomic.rmw16.sub_u"
        | b"i64.atomic.rmw16.sub_u"
        | b"i32.atomic.rmw16.and_u"
        | b"i64.atomic.rmw16.and_u"
        | b"i32.atomic.rmw16.or_u"
        | b"i64.atomic.rmw16.or_u"
        | b"i32.atomic.rmw16.xor_u"
        | b"i64.atomic.rmw16.xor_u"
        | b"i32.atomic.rmw16.xchg_u"
        | b"i64.atomic.rmw16.xchg_u"
        | b"i32.atomic.rmw16.cmpxchg_u"
        | b"i64.atomic.rmw16.cmpxchg_u" => isa::Align::Two,
        b"i32.load"
        | b"f32.load"
        | b"i32.store"
        | b"f32.store"
        | b"i64.load32_s"
        | b"i64.load32_u"
        | b"i64.store32"
        | b"v128.load32_splat"
        | b"v128.load32_zero"
        | b"v128.load32_lane"
        | b"v128.store32_lane"
        | b"memory.atomic.notify"
        | b"memory.atomic.wait32"
        | b"i32.atomic.load"
        | b"i64.atomic.load32_u"
        | b"i32.atomic.store"
        | b"i64.atomic.store32_u"
        | b"i32.atomic.rmw.add"
        | b"i64.atomic.rmw32.add_u"
        | b"i32.atomic.rmw.sub"
        | b"i64.atomic.rmw32.sub_u"
        | b"i32.atomic.rmw.and"
        | b"i64.atomic.rmw32.and_u"
        | b"i32.atomic.rmw.or"
        | b"i64.atomic.rmw32.or_u"
        | b"i32.atomic.rmw.xor"
        | b"i64.atomic.rmw32.xor_u"
        | b"i32.atomic.rmw.xchg"
        | b"i64.atomic.rmw32.xchg_u"
        | b"i32.atomic.rmw.cmpxchg"
        | b"i64.atomic.rmw32.cmpxchg_u" => isa::Align::Four,
        b"i64.load"
        | b"f64.load"
        | b"i64.store"
        | b"f64.store"
        | b"v128.load8x8_s"
        | b"v128.load8x8_u"
        | b"v128.load16x4_s"
        | b"v128.load16x4_u"
        | b"v128.load32x2_s"
        | b"v128.load32x2_u"
        | b"v128.load64_splat"
        | b"v128.load64_zero"
        | b"v128.load64_lane"
        | b"v128.store64_lane"
        | b"memory.atomic.wait64"
        | b"i64.atomic.load"
        | b"i64.atomic.store"
        | b"i64.atomic.rmw.add"
        | b"i64.atomic.rmw.sub"
        | b"i64.atomic.rmw.and"
        | b"i64.atomic.rmw.or"
        | b"i64.atomic.rmw.xor"
        | b"i64.atomic.rmw.xchg"
        | b"i64.atomic.rmw.cmpxchg" => isa::Align::Eight,
        b"v128.load" | b"v128.store" => isa::Align::Sixteen,
        _ => panic!("{}", name),
    }
}

fn display_mem_arg(
    default_alignment: isa::Align,
    arg: MemArg,
    f: &mut Formatter,
) -> core::fmt::Result {
    if arg.memory != MemIdx(0) {
        write!(f, "{}", arg.memory)?;
    }

    if arg.offset != 0 {
        write!(f, " offset={}", arg.offset)?;
    }

    if arg.align == default_alignment {
        write!(f, " align={}", arg.align.in_bytes())?;
    }

    Ok(())
}

macro_rules! instr_case_common_display {
    ($pascal_ident:ident) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)
            }
        }
    };
    ($pascal_ident:ident { block_type: BlockType }) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                match self.block_type {
                    BlockType::Empty => Ok(()),
                    BlockType::Index(idx) => write!(f, " (type {idx})"),
                    BlockType::Inline(ty) => write!(f, " (result {ty})"),
                }
            }
        }
    };
    ($pascal_ident:ident { arg: MemArg }) => {
        impl<A: Allocator> $pascal_ident<A> {
            const NATURAL_ALIGN: isa::Align = mem_arg_natural_align(Self::NAME);
        }

        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                display_mem_arg(Self::NATURAL_ALIGN, self.arg, f)
            }
        }
    };
    ($pascal_ident:ident { n: $integer_type:ident }) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                let width = 2 + (<$integer_type>::BITS as usize / 4);
                write!(f, " {:#0width$X} (* signed = {}, unsigned = {} *)", self.n, self.n as i32, self.n)
            }
        }
    };
    ($pascal_ident:ident { z: $_float_type:ident }) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                write!(f, " {:?} (* {:.} *)", self.z, self.z.interpret())
            }
        }
    };
    ($pascal_ident:ident { arg: MemArg, lane: LaneIdx }) => {
        impl<A: Allocator> $pascal_ident<A> {
            const NATURAL_ALIGN: isa::Align = mem_arg_natural_align(Self::NAME);
        }

        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                display_mem_arg(Self::NATURAL_ALIGN, self.arg, f)?;
                write!(f, "{}", self.lane)
            }
        }
    };
    ($pascal_ident:ident { v: V128 }) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                f.write_str(" i8x16")?;
                for b in self.v.0 {
                    write!(f, " {b:#04X}")?;
                }
                Ok(())
            }
        }
    };
    ($pascal_ident:ident { lanes: V128ShuffleLanes }) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                for idx in self.lanes.0 {
                    write!(f, " {idx:#04X}")?;
                }
                Ok(())
            }
        }
    };
    ($pascal_ident:ident { $($field_name:ident: $_field_type:ident),+ }) => {
        impl<A: Allocator> Display for $pascal_ident<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str(Self::NAME)?;
                $(
                    write!(f, " {}", self.$field_name)?;
                )+
                Ok(())
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
    (ByteOpcode $wasm_name:literal BrTable { targets: BrTableTargets }) => {
        instr_case_common!(ByteOpcode $wasm_name BrTable);
    };
    (ByteOpcode $wasm_name:literal SelectTyped { types: SelectTypes }) => {
        instr_case_common!(ByteOpcode $wasm_name SelectTyped);
    };
    {
        $opcode_enum:ident $wasm_name:literal $pascal_ident:ident $({
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

        instr_case_common!($opcode_enum $wasm_name $pascal_ident);

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
        instr_case_common_display!($pascal_ident $({$($field_name: $field_type),+})?);
    };
}

#[derive(Clone)]
#[allow(missing_docs)]
#[non_exhaustive]
pub struct BrTable<A: Allocator = Global> {
    pub targets: Box<[LabelIdx], A>,
    pub default_target: LabelIdx,
}

impl<A1: Allocator, A2: Allocator> PartialEq<BrTable<A2>> for BrTable<A1> {
    #[inline]
    fn eq(&self, other: &BrTable<A2>) -> bool {
        let self_targets: &[LabelIdx] = &self.targets;
        let other_targets: &[LabelIdx] = &other.targets;
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

impl<A: Allocator> Display for BrTable<A> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str(Self::NAME)?;
        for idx in self.targets.iter() {
            write!(f, " {idx}")?;
        }
        write!(f, " {}", self.default_target)
    }
}

#[derive(Clone)]
#[allow(missing_docs)]
#[non_exhaustive]
pub struct SelectTyped<A: Allocator = Global> {
    operand_type: ValType,
    _marker: PhantomData<fn() -> A>,
}

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

impl<A: Allocator> Display for SelectTyped<A> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_str(Self::NAME)?;
        write!(f, " (result {})", self.operand_type)
    }
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

impl<A: Allocator, P: ParseExpr<A>> ParseExpr<A> for &mut P {
    #[inline]
    fn parse(&mut self, instr: Instr<A>) -> Result<(), UnrecognizedInstr> {
        P::parse(self, instr)
    }
}

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

impl<'a, E, P> Parser<'a, E, P>
where
    E: ErrorSource<'a>,
    P: ParseExpr,
{
    #[allow(missing_docs)]
    #[inline]
    pub fn new(parser: P) -> Self {
        Self::with_allocator(parser, Global)
    }
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
        let mut other_targets =
            Vec::with_capacity_in(targets.expected_len() - 1, self.allocator.clone());
        let mut default_target = LabelIdx(0);
        while let Some(label) = crate::values::Sequence::parse(targets)? {
            if targets.expected_len() == 0 {
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
        let operand_type = crate::values::Sequence::parse(types)
            .transpose()
            .expect("SelectTypes implementation always returns at least 1 type")?;

        if types.expected_len() > 0 {
            let arity = u8::try_from(types.expected_len())
                .ok()
                .and_then(|a| a.checked_add(1))
                .and_then(core::num::NonZeroU8::new)
                .unwrap_or(core::num::NonZeroU8::MAX);

            let e = E::from_error_cause(
                start,
                crate::error::ErrorCause::Instr {
                    opcode: Opcode::SelectTyped,
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

            pub fn opcode(&self) -> Opcode {
                match self {
                    $(Self::$pascal_ident(_) => Opcode::$pascal_ident,)*
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

        impl<A: Allocator> Display for Instr<A> {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                match self {
                    $(Self::$pascal_ident(ident) => Display::fmt(ident, f),)*
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
