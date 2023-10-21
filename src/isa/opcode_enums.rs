use crate::{
    isa::{InvalidOpcode, Opcode},
    static_assert,
};

macro_rules! opcode_methods {
    {
        $enum_name:ident => {
            $($case_name:ident = $case_value:literal,)*
        }
    } => {
        #[allow(missing_docs)]
        impl $enum_name {
            pub const fn to_opcode(self) -> Opcode {
                match self {
                    $(Self::$case_name => Opcode::$case_name,)*
                }
            }

            pub const fn from_opcode(opcode: Opcode) -> Option<Self> {
                match opcode {
                    $(Opcode::$case_name => Some(Self::$case_name),)*
                    _ => None,
                }
            }

            #[inline]
            pub const fn name(self) -> &'static str {
                self.to_opcode().name()
            }
        }

        impl From<$enum_name> for Opcode {
            #[inline]
            fn from(opcode: $enum_name) -> Self {
                opcode.to_opcode()
            }
        }

        impl core::fmt::Display for $enum_name {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str(self.name())
            }
        }
    };
}

macro_rules! byte_opcodes {
    {
        $(
            $(#[$case_meta:meta])*
            $case_name:ident = $case_value:literal,
        )*
    } => {
        crate::tag::enumeration_basic! {
            /// List of all WebAssembly [*opcodes*] that encode an instruction in a single byte.
            ///
            /// See the [`Opcode`] `enum` for the set of all possible WebAssembly instruction
            /// [*opcodes*], including those with prefix bytes.
            ///
            /// [*opcodes*]: https://webassembly.github.io/spec/core/binary/instructions.html#instructions
            #[derive(Default)]
            #[non_exhaustive]
            #[allow(missing_docs)]
            pub ByteOpcode : u8 {
                $(
                    $(#[$case_meta])*
                    $case_name = $case_value,
                )*
            }
        }

        opcode_methods!(ByteOpcode => { $($case_name = $case_value,)* });
    };
}

macro_rules! prefixed_opcodes {
    ($(
        $(#[$enum_meta:meta])*
        $enum_name:ident($prefix:literal) {
            $($name:ident = $value:literal,)*
        }
    )*) => {$(
        crate::tag::enumeration_basic! {
            $(#[$enum_meta])*
            #[non_exhaustive]
            pub $enum_name : u32 {
                $(
                    #[allow(missing_docs)]
                    $name = $value,
                )*
            }
        }

        impl $enum_name {
            #[allow(missing_docs)]
            pub const PREFIX: u8 = $prefix;
        }

        impl TryFrom<u32> for $enum_name {
            type Error = InvalidOpcode;

            fn try_from(opcode: u32) -> Result<Self, Self::Error> {
                Self::new(opcode).ok_or_else(|| InvalidOpcode::new($prefix, Some(opcode)))
            }
        }

        opcode_methods!($enum_name => { $($name = $value,)* });
    )*};
}

byte_opcodes! {
    #[default]
    Unreachable = 0x00,
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    Try = 0x06,
    Catch = 0x07,
    Throw = 0x08,
    Rethrow = 0x09,
    End = 0x0B,
    Br = 0x0C,
    BrIf = 0x0D,
    BrTable = 0x0E,
    Return = 0x0F,
    Call = 0x10,
    CallIndirect = 0x11,
    ReturnCall = 0x12,
    ReturnCallIndirect = 0x13,
    Delegate = 0x18,
    CatchAll = 0x19,

    Drop = 0x1A,
    Select = 0x1B,
    /// Alternative opcode for [`select`](Opcode::Select), used to explicitly specify the types of
    /// operands.
    SelectTyped = 0x1C,

    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,

    GlobalGet = 0x23,
    GlobalSet = 0x24,

    TableGet = 0x25,
    TableSet = 0x26,

    I32Load = 0x28,
    I64Load = 0x29,
    F32Load = 0x2A,
    F64Load = 0x2B,

    I32Load8S = 0x2C,
    I32Load8U = 0x2D,
    I32Load16S = 0x2E,
    I32Load16U = 0x2F,
    I64Load8S = 0x30,
    I64Load8U = 0x31,
    I64Load16S = 0x32,
    I64Load16U = 0x33,
    I64Load32S = 0x34,
    I64Load32U = 0x35,

    I32Store = 0x36,
    I64Store = 0x37,
    F32Store = 0x38,
    F64Store = 0x39,

    I32Store8 = 0x3A,
    I32Store16 = 0x3B,
    I64Store8 = 0x3C,
    I64Store16 = 0x3D,
    I64Store32 = 0x3E,

    MemorySize = 0x3F,
    MemoryGrow = 0x40,

    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,

    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,

    I64Eqz = 0x50,
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5A,

    F32Eq = 0x5B,
    F32Ne = 0x5C,
    F32Lt = 0x5D,
    F32Gt = 0x5E,
    F32Le = 0x5F,
    F32Ge = 0x60,
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,

    I32Clz = 0x67,
    I32Ctz = 0x68,
    I32Popcnt = 0x69,
    I32Add = 0x6A,
    I32Sub = 0x6B,
    I32Mul = 0x6C,
    I32DivS = 0x6D,
    I32DivU = 0x6E,
    I32RemS = 0x6F,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    I32Rotl = 0x77,
    I32Rotr = 0x78,

    I64Clz = 0x79,
    I64Ctz = 0x7A,
    I64Popcnt = 0x7B,
    I64Add = 0x7C,
    I64Sub = 0x7D,
    I64Mul = 0x7E,
    I64DivS = 0x7F,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64Shl = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,
    I64Rotl = 0x89,
    I64Rotr = 0x8A,

    F32Abs = 0x8B,
    F32Neg = 0x8C,
    F32Ceil = 0x8D,
    F32Floor = 0x8E,
    F32Trunc = 0x8F,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,
    F32Copysign = 0x98,

    F64Abs = 0x99,
    F64Neg = 0x9A,
    F64Ceil = 0x9B,
    F64Floor = 0x9C,
    F64Trunc = 0x9D,
    F64Nearest = 0x9E,
    F64Sqrt = 0x9F,
    F64Add = 0xA0,
    F64Sub = 0xA1,
    F64Mul = 0xA2,
    F64Div = 0xA3,
    F64Min = 0xA4,
    F64Max = 0xA5,
    F64Copysign = 0xA6,

    I32WrapI64 = 0xA7,
    I32TruncF32S = 0xA8,
    I32TruncF32U = 0xA9,
    I32TruncF64S = 0xAA,
    I32TruncF64U = 0xAB,
    I64ExtendI32S = 0xAC,
    I64ExtendI32U = 0xAD,
    I64TruncF32S = 0xAE,
    I64TruncF32U = 0xAF,
    I64TruncF64S = 0xB0,
    I64TruncF64U = 0xB1,
    F32ConvertI32S = 0xB2,
    F32ConvertI32U = 0xB3,
    F32ConvertI64S = 0xB4,
    F32ConvertI64U = 0xB5,
    F32DemoteF64 = 0xB6,
    F64ConvertI32S = 0xB7,
    F64ConvertI32U = 0xB8,
    F64ConvertI64S = 0xB9,
    F64ConvertI64U = 0xBA,
    F64PromoteF32 = 0xBB,
    I32ReinterpretF32 = 0xBC,
    I64ReinterpretF64 = 0xBD,
    F32ReinterpretI32 = 0xBE,
    F64ReinterpretI64 = 0xBF,

    I32Extend8S = 0xC0,
    I32Extend16S = 0xC1,
    I64Extend8S = 0xC2,
    I64Extend16S = 0xC3,
    I64Extend32S = 0xC4,

    RefNull = 0xD0,
    RefIsNull = 0xD1,
    RefFunc = 0xD2,
}

prefixed_opcodes! {
    /// An opcode value for an instruction prefixed by the `0xFC` [`Opcode`].
    ///
    /// The feature proposals that introduced these opcodes include:
    /// - The [non-trapping float-to-integer conversions] proposal.
    /// - The [bulk-memory operations] proposal.
    /// - The [reference types] proposal.
    ///
    /// [non-trapping float-to-integer conversions]: https://github.com/WebAssembly/nontrapping-float-to-int-conversions
    /// [bulk-memory operations]: https://github.com/WebAssembly/bulk-memory-operations
    /// [reference types]: https://github.com/WebAssembly/reference-types
    FCPrefixedOpcode(0xFC) {
        I32TruncSatF32S = 0,
        I32TruncSatF32U = 1,
        I32TruncSatF64S = 2,
        I32TruncSatF64U = 3,
        I64TruncSatF32S = 4,
        I64TruncSatF32U = 5,
        I64TruncSatF64S = 6,
        I64TruncSatF64U = 7,

        MemoryInit = 8,
        DataDrop = 9,
        MemoryCopy = 10,
        MemoryFill = 11,

        TableInit = 12,
        ElemDrop = 13,
        TableCopy = 14,
        TableGrow = 15,
        TableSize = 16,
        TableFill = 17,
    }

    /// An opcode value for an instruction prefixed by the `0xFE` ``Opcode`].
    ///
    /// The feature proposals that introduced these opcodes include:
    /// - The [threads] proposal, which introduced atomic memory instructions.
    ///
    /// [threads]: https://github.com/webassembly/threads
    FEPrefixedOpcode(0xFE) {
        MemoryAtomicNotify = 0,
        MemoryAtomicWait32 = 1,
        MemoryAtomicWait64 = 2,
        //AtomicFence = 3,

        I32AtomicLoad = 0x10,
        I64AtomicLoad = 0x11,
        I32AtomicLoad8U = 0x12,
        I32AtomicLoad16U = 0x13,
        I64AtomicLoad8U = 0x14,
        I64AtomicLoad16U = 0x15,
        I64AtomicLoad32U = 0x16,

        I32AtomicStore = 0x17,
        I64AtomicStore = 0x18,
        I32AtomicStore8U = 0x19,
        I32AtomicStore16U = 0x1A,
        I64AtomicStore8U = 0x1B,
        I64AtomicStore16U = 0x1C,
        I64AtomicStore32U = 0x1D,

        I32AtomicRmwAdd = 0x1E,
        I64AtomicRmwAdd = 0x1F,
        I32AtomicRmw8AddU = 0x20,
        I32AtomicRmw16AddU = 0x21,
        I64AtomicRmw8AddU = 0x22,
        I64AtomicRmw16AddU = 0x23,
        I64AtomicRmw32AddU = 0x24,

        I32AtomicRmwSub = 0x25,
        I64AtomicRmwSub = 0x26,
        I32AtomicRmw8SubU = 0x27,
        I32AtomicRmw16SubU = 0x28,
        I64AtomicRmw8SubU = 0x29,
        I64AtomicRmw16SubU = 0x2A,
        I64AtomicRmw32SubU = 0x2B,

        I32AtomicRmwAnd = 0x2C,
        I64AtomicRmwAnd = 0x2D,
        I32AtomicRmw8AndU = 0x2E,
        I32AtomicRmw16AndU = 0x2F,
        I64AtomicRmw8AndU = 0x30,
        I64AtomicRmw16AndU = 0x31,
        I64AtomicRmw32AndU = 0x32,

        I32AtomicRmwOr = 0x33,
        I64AtomicRmwOr = 0x34,
        I32AtomicRmw8OrU = 0x35,
        I32AtomicRmw16OrU = 0x36,
        I64AtomicRmw8OrU = 0x37,
        I64AtomicRmw16OrU = 0x38,
        I64AtomicRmw32OrU = 0x39,

        I32AtomicRmwXor = 0x3A,
        I64AtomicRmwXor = 0x3B,
        I32AtomicRmw8XorU = 0x3C,
        I32AtomicRmw16XorU = 0x3D,
        I64AtomicRmw8XorU = 0x3E,
        I64AtomicRmw16XorU = 0x3F,
        I64AtomicRmw32XorU = 0x40,

        I32AtomicRmwXchg = 0x41,
        I64AtomicRmwXchg = 0x42,
        I32AtomicRmw8XchgU = 0x43,
        I32AtomicRmw16XchgU = 0x44,
        I64AtomicRmw8XchgU = 0x45,
        I64AtomicRmw16XchgU = 0x46,
        I64AtomicRmw32XchgU = 0x47,

        I32AtomicRmwCmpxchg = 0x48,
        I64AtomicRmwCmpxchg = 0x49,
        I32AtomicRmw8CmpxchgU = 0x4A,
        I32AtomicRmw16CmpxchgU = 0x4B,
        I64AtomicRmw8CmpxchgU = 0x4C,
        I64AtomicRmw16CmpxchgU = 0x4D,
        I64AtomicRmw32CmpxchgU = 0x4E,
    }

    /// An opcode value for a [128-bit vector instruction], which is an instruction prefixed by the
    /// `0xFD` [`Opcode`].
    ///
    /// The feature proposals that introduced these opcodes include:
    /// - The [fixed-width SIMD proposal], which introduced the [`0xFD` opcode] prefix.
    /// - The [relaxed SIMD proposal], which introduced additional opcodes on top of the
    ///   [fixed-width SIMD proposal].
    ///
    /// [128-bit vector instruction]: https://webassembly.github.io/spec/core/binary/instructions.html#vector-instructions
    /// [fixed-width SIMD proposal]: https://github.com/webassembly/simd
    /// [relaxed SIMD proposal]: https://github.com/WebAssembly/relaxed-simd
    V128Opcode(0xFD) {
        V128Load = 0,
        V128Load8x8S = 1,
        V128Load8x8U = 2,
        V128Load16x4S = 3,
        V128Load16x4U = 4,
        V128Load32x2S = 5,
        V128Load32x2U = 6,
        V128Load8Splat = 7,
        V128Load16Splat = 8,
        V128Load32Splat = 9,
        V128Load64Splat = 10,
        V128Store = 11,

        V128Const = 12,

        I8x16Shuffle = 13,

        I8x16Swizzle = 14,
        I8x16Splat = 15,
        I16x8Splat = 16,
        I32x4Splat = 17,
        I64x2Splat = 18,
        F32x4Splat = 19,
        F64x2Splat = 20,

        I8x16ExtractLaneS = 21,
        I8x16ExtractLaneU = 22,
        I8x16ReplaceLane = 23,
        I16x8ExtractLaneS = 24,
        I16x8ExtractLaneU = 25,
        I16x8ReplaceLane = 26,
        I32x4ExtractLane = 27,
        I32x4ReplaceLane = 28,
        I64x2ExtractLane = 29,
        I64x2ReplaceLane = 30,
        F32x4ExtractLane = 31,
        F32x4ReplaceLane = 32,
        F64x2ExtractLane = 33,
        F64x2ReplaceLane = 34,

        I8x16Eq = 35,
        I8x16Ne = 36,
        I8x16LtS = 37,
        I8x16LtU = 38,
        I8x16GtS = 39,
        I8x16GtU = 40,
        I8x16LeS = 41,
        I8x16LeU = 42,
        I8x16GeS = 43,
        I8x16GeU = 44,

        I16x8Eq = 45,
        I16x8Ne = 46,
        I16x8LtS = 47,
        I16x8LtU = 48,
        I16x8GtS = 49,
        I16x8GtU = 50,
        I16x8LeS = 51,
        I16x8LeU = 52,
        I16x8GeS = 53,
        I16x8GeU = 54,

        I32x4Eq = 55,
        I32x4Ne = 56,
        I32x4LtS = 57,
        I32x4LtU = 58,
        I32x4GtS = 59,
        I32x4GtU = 60,
        I32x4LeS = 61,
        I32x4LeU = 62,
        I32x4GeS = 63,
        I32x4GeU = 64,

        F32x4Eq = 65,
        F32x4Ne = 66,
        F32x4Lt = 67,
        F32x4Gt = 68,
        F32x4Le = 69,
        F32x4Ge = 70,

        F64x2Eq = 71,
        F64x2Ne = 72,
        F64x2Lt = 73,
        F64x2Gt = 74,
        F64x2Le = 75,
        F64x2Ge = 76,

        V128Not = 77,
        V128And = 78,
        V128AndNot = 79,
        V128Or = 80,
        V128Xor = 81,
        V128Bitselect = 82,
        V128AnyTrue = 83,

        V128Load8Lane = 84,
        V128Load16Lane = 85,
        V128Load32Lane = 86,
        V128Load64Lane = 87,
        V128Store8Lane = 88,
        V128Store16Lane = 89,
        V128Store32Lane = 90,
        V128Store64Lane = 91,
        V128Load32Zero = 92,
        V128Load64Zero = 93,

        F32x4DemoteF64x2Zero = 94,
        F64x2PromoteLowF32x4 = 95,

        I8x16Abs = 96,
        I8x16Neg = 97,
        I8x16Popcnt = 98,
        I8x16AllTrue = 99,
        I8x16Bitmask = 100,
        I8x16NarrowI16x8S = 101,
        I8x16NarrowI16x8U = 102,

        F32x4Ceil = 103,
        F32x4Floor = 104,
        F32x4Trunc = 105,
        F32x4Nearest = 106,

        I8x16Shl = 107,
        I8x16ShrS = 108,
        I8x16ShrU = 109,
        I8x16Add = 110,
        I8x16AddSatS = 111,
        I8x16AddSatU = 112,
        I8x16Sub = 113,
        I8x16SubSatS = 114,
        I8x16SubSatU = 115,

        F64x2Ceil = 116,
        F64x2Floor = 117,

        I8x16MinS = 118,
        I8x16MinU = 119,
        I8x16MaxS = 120,
        I8x16MaxU = 121,

        F64x2Trunc = 122,

        I8x16AvgrU = 123,

        I16x8ExtaddPairwiseI8x16S = 124,
        I16x8ExtaddPairwiseI8x16U = 125,
        I32x4ExtaddPairwiseI16x8S = 126,
        I32x4ExtaddPairwiseI16x8U = 127,

        I16x8Abs = 128,
        I16x8Neg = 129,
        I16x8Q15mulrSatS = 130,
        I16x8AllTrue = 131,
        I16x8Bitmask = 132,
        I16x8NarrowI32x4S = 133,
        I16x8NarrowI32x4U = 134,
        I16x8ExtendLowI8x16S = 135,
        I16x8ExtendHighI8x16S = 136,
        I16x8ExtendLowI8x16U = 137,
        I16x8ExtendHighI8x16U = 138,
        I16x8Shl = 139,
        I16x8ShrS = 140,
        I16x8ShrU = 141,
        I16x8Add = 142,
        I16x8AddSatS = 143,
        I16x8AddSatU = 144,
        I16x8Sub = 145,
        I16x8SubSatS = 146,
        I16x8SubSatU = 147,

        F64x2Nearest = 148,

        I16x8Mul = 149,
        I16x8MinS = 150,
        I16x8MinU = 151,
        I16x8MaxS = 152,
        I16x8MaxU = 153,
        I16x8AvgrU = 155,
        I16x8ExtmulLowI8x16S = 156,
        I16x8ExtmulHighI8x16S = 157,
        I16x8ExtmulLowI8x16U = 158,
        I16x8ExtmulHighI8x16U = 159,

        I32x4Abs = 160,
        I32x4Neg = 161,
        I32x4AllTrue = 163,
        I32x4Bitmask = 164,
        I32x4ExtendLowI16x8S = 167,
        I32x4ExtendHighI16x8S = 168,
        I32x4ExtendLowI16x8U = 169,
        I32x4ExtendHighI16x8U = 170,
        I32x4Shl = 171,
        I32x4ShrS = 172,
        I32x4ShrU = 173,
        I32x4Add = 174,
        I32x4Sub = 177,
        I32x4Mul = 181,
        I32x4MinS = 182,
        I32x4MinU = 183,
        I32x4MaxS = 184,
        I32x4MaxU = 185,
        I32x4DotI16x8S = 186,
        I32x4ExtmulLowI16x8S = 188,
        I32x4ExtmulHighI16x8S = 189,
        I32x4ExtmulLowI16x8U = 190,
        I32x4ExtmulHighI16x8U = 191,

        I64x2Abs = 192,
        I64x2Neg = 193,
        I64x2AllTrue = 195,
        I64x2Bitmask = 196,
        I64x2ExtendLowI32x4S = 199,
        I64x2ExtendHighI32x4S = 200,
        I64x2ExtendLowI32x4U = 201,
        I64x2ExtendHighI32x4U = 202,
        I64x2Shl = 203,
        I64x2ShrS = 204,
        I64x2ShrU = 205,
        I64x2Add = 206,
        I64x2Sub = 209,
        I64x2Mul = 213,

        I64x2Eq = 214,
        I64x2Ne = 215,
        I64x2LtS = 216,
        I64x2GtS = 217,
        I64x2LeS = 218,
        I64x2GeS = 219,

        I64x2ExtmulLowI32x4S = 220,
        I64x2ExtmulHighI32x4S = 221,
        I64x2ExtmulLowI32x4U = 222,
        I64x2ExtmulHighI32x4U = 223,

        F32x4Abs = 224,
        F32x4Neg = 225,
        F32x4Sqrt = 227,
        F32x4Add = 228,
        F32x4Sub = 229,
        F32x4Mul = 230,
        F32x4Div = 231,
        F32x4Min = 232,
        F32x4Max = 233,
        F32x4Pmin = 234,
        F32x4Pmax = 235,

        F64x2Abs = 236,
        F64x2Neg = 237,
        F64x2Sqrt = 239,
        F64x2Add = 240,
        F64x2Sub = 241,
        F64x2Mul = 242,
        F64x2Div = 243,
        F64x2Min = 244,
        F64x2Max = 245,
        F64x2Pmin = 246,
        F64x2Pmax = 247,

        I32x4TruncSatF32x4S = 248,
        I32x4TruncSatF32x4U = 249,
        F32x4ConvertI32x4S = 250,
        F32x4ConvertI32x4U = 251,
        I32x4TruncSatF64x2SZero = 252,
        I32x4TruncSatF64x2UZero = 253,
        F64x2ConvertLowI32x4S = 254,
        F64x2ConvertLowI32x4U = 255,

        I8x16RelaxedSwizzle = 0x100,
        I32x4RelaxedTruncF32x4S = 0x101,
        I32x4RelaxedTruncF32x4U = 0x102,
        I32x4RelaxedTruncF64x2SZero = 0x103,
        I32x4RelaxedTruncF64x2UZero = 0x104,
        F32x4RelaxedMadd = 0x105, // multiply-add
        F32x4RelaxedNmadd = 0x106, // negative multiply-add
        F64x2RelaxedMadd = 0x107,
        F64x2RelaxedNmadd = 0x108,
        I8x16RelaxedLaneselect = 0x109,
        I16x8RelaxedLaneselect = 0x10A,
        I32x4RelaxedLaneselect = 0x10B,
        I64x2RelaxedLaneselect = 0x10C,
        F32x4RelaxedMin = 0x10D,
        F32x4RelaxedMax = 0x10E,
        F64x2RelaxedMin = 0x10F,
        F64x2RelaxedMax = 0x110,
        I16x8RelaxedQ15mulrS = 0x111,
        I16x8RelaxedDotI8x16I7x16S = 0x112,
        I32x4RelaxedDotI8x16I7x16AddS = 0x113,

        // Relaxed SIMD Reserved Range (0x114 - 0x12F)
    }
}

static_assert::check_size!(ByteOpcode, <= 1);
static_assert::check_size!(FCPrefixedOpcode, <= 1);
static_assert::check_size!(FEPrefixedOpcode, <= 1);
static_assert::check_size!(V128Opcode, <= 2);

impl TryFrom<u8> for ByteOpcode {
    type Error = InvalidOpcode;

    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        ByteOpcode::new(opcode).ok_or_else(|| InvalidOpcode::new(opcode, None))
    }
}
