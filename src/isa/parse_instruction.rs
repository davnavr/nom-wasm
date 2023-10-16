use crate::{
    error::{AddCause as _, ErrorCause, ErrorKind, ErrorSource, InvalidInstr},
    index::Index as _,
    isa::{self, FCPrefixedOpcode, FEPrefixedOpcode, InstrKind, Opcode, ParseInstr, V128Opcode},
    module::{self, MemIdx, TableIdx, TypeIdx},
};

trait ResultExt<'a, T, E: ErrorSource<'a>> {
    fn to_parsed(self, input: &&'a [u8], opcode: InstrKind) -> crate::input::Result<T, E>;
}

impl<'a, T, E: ErrorSource<'a>> ResultExt<'a, T, E> for isa::Result<T, E> {
    #[inline]
    fn to_parsed(self, start: &&'a [u8], opcode: InstrKind) -> crate::input::Result<T, E> {
        match self {
            Ok(value) => Ok(value),
            Err(isa::ParseInstrError::Unrecognized) => {
                Err(nom::Err::Failure(E::from_error_kind_and_cause(
                    start,
                    ErrorKind::Verify,
                    ErrorCause::Instr {
                        opcode,
                        reason: InvalidInstr::Unrecognized,
                    },
                )))
            }
            Err(isa::ParseInstrError::ParseFailed(err)) => Err(nom::Err::Failure(err)),
        }
    }
}

/// Parses a [WebAssembly **`instr`**uction].
///
/// [WebAssembly **`instr`**uction]: https://webassembly.github.io/spec/core/binary/instructions.html
pub fn instr<'a, P, E>(input: &'a [u8], mut parser: P) -> crate::Parsed<'a, P, E>
where
    P: ParseInstr<'a, E>,
    E: ErrorSource<'a>,
{
    let start = &input;
    let (input, opcode) = InstrKind::parse(input)?;

    let bad_instr = move |reason| ErrorCause::Instr { opcode, reason };
    let bad_argument = move || bad_instr(InvalidInstr::Argument);

    let parse_lane_idx = move |input: &'a [u8]| -> crate::Parsed<'a, isa::LaneIdx, E> {
        if let Some((lane, input)) = input.split_first() {
            Ok((input, *lane))
        } else {
            Err(nom::Err::Failure(E::from_error_kind_and_cause(
                input,
                ErrorKind::Eof,
                bad_instr(InvalidInstr::VectorLane),
            )))
        }
    };

    macro_rules! empty_case {
        ($case:ident) => {{
            parser.$case().to_parsed(start, opcode)?;
            input
        }};
    }

    macro_rules! simple_arguments {
        ($($parameter:ident: $argument:ty),+ => $case:ident) => {{
            $(
                let (input, $parameter) = <$argument>::parse(input).add_cause_with(bad_argument)?;
            )+

            parser.$case($($parameter),+).to_parsed(start, opcode)?;
            input
        }};
    }

    macro_rules! single_argument {
        ($argument:ty => $case:ident) => {
            simple_arguments!(arg: $argument => $case)
        };
    }

    macro_rules! block_start {
        ($case:ident) => {
            single_argument!(crate::types::BlockType => $case)
        };
    }

    macro_rules! mem_op {
        ($case:ident) => {
            single_argument!(isa::MemArg => $case)
        };
    }

    macro_rules! copy_op {
        ($index:ty => $case:ident) => {{
            let (input, destination) = <$index>::parse(input)
                .add_cause_with(move || bad_instr(InvalidInstr::Destination))?;

            let (input, source) =
                <$index>::parse(input).add_cause_with(move || bad_instr(InvalidInstr::Source))?;

            parser.$case(destination, source).to_parsed(start, opcode)?;
            input
        }};
    }

    macro_rules! v128_mem_lane_op {
        ($case:ident) => {{
            let (input, memarg) = isa::MemArg::parse(input).add_cause_with(bad_argument)?;
            let (input, lane) = parse_lane_idx(input)?;
            parser.$case(memarg, lane).to_parsed(start, opcode)?;
            input
        }};
    }

    macro_rules! v128_lane_op {
        ($case:ident) => {{
            let (input, lane) = parse_lane_idx(input)?;
            parser.$case(lane).to_parsed(start, opcode)?;
            input
        }};
    }

    let input = match opcode {
        kind @ InstrKind::Byte(opcode) => match opcode {
            Opcode::Unreachable => empty_case!(unreachable),
            Opcode::Nop => empty_case!(nop),
            Opcode::Block => block_start!(block),
            Opcode::Loop => block_start!(r#loop),
            Opcode::If => block_start!(r#if),
            Opcode::Else => empty_case!(r#else),
            Opcode::End => empty_case!(end),
            Opcode::Br => single_argument!(isa::LabelIdx => br),
            Opcode::BrIf => single_argument!(isa::LabelIdx => br_if),
            Opcode::BrTable => {
                let mut targets =
                    isa::BrTableTargets::with_input(input).add_cause_with(bad_argument)?;

                parser.br_table(&mut targets).to_parsed(start, kind)?;
                targets.finish().add_cause_with(bad_argument)?.0
            }
            Opcode::Return => empty_case!(r#return),
            Opcode::Call => single_argument!(module::FuncIdx => call),
            Opcode::CallIndirect => {
                simple_arguments!(signature: TypeIdx, table: TableIdx => call_indirect)
            }
            Opcode::Drop => empty_case!(drop),
            Opcode::Select => empty_case!(select),
            Opcode::SelectTyped => {
                let mut types = isa::SelectTypes::with_parsed_length(input, Default::default())
                    .add_cause_with(bad_argument)?;

                parser.select_typed(&mut types).to_parsed(start, kind)?;
                types.finish().add_cause_with(bad_argument)?.0
            }
            Opcode::LocalGet => single_argument!(module::LocalIdx => local_get),
            Opcode::LocalSet => single_argument!(module::LocalIdx => local_set),
            Opcode::LocalTee => single_argument!(module::LocalIdx => local_tee),
            Opcode::GlobalGet => single_argument!(module::GlobalIdx => global_get),
            Opcode::GlobalSet => single_argument!(module::GlobalIdx => global_set),
            Opcode::I32Load => mem_op!(i32_load),
            Opcode::I64Load => mem_op!(i64_load),
            Opcode::F32Load => mem_op!(f32_load),
            Opcode::F64Load => mem_op!(f64_load),
            Opcode::I32Load8S => mem_op!(i32_load8_s),
            Opcode::I32Load8U => mem_op!(i32_load8_u),
            Opcode::I32Load16S => mem_op!(i32_load16_s),
            Opcode::I32Load16U => mem_op!(i32_load16_u),
            Opcode::I64Load8S => mem_op!(i64_load8_s),
            Opcode::I64Load8U => mem_op!(i64_load8_u),
            Opcode::I64Load16S => mem_op!(i64_load16_s),
            Opcode::I64Load16U => mem_op!(i64_load16_u),
            Opcode::I64Load32S => mem_op!(i64_load32_s),
            Opcode::I64Load32U => mem_op!(i64_load32_u),
            Opcode::I32Store => mem_op!(i32_store),
            Opcode::I64Store => mem_op!(i64_store),
            Opcode::F32Store => mem_op!(f32_store),
            Opcode::F64Store => mem_op!(f64_store),
            Opcode::I32Store8 => mem_op!(i32_store8),
            Opcode::I32Store16 => mem_op!(i32_store16),
            Opcode::I64Store8 => mem_op!(i64_store8),
            Opcode::I64Store16 => mem_op!(i64_store16),
            Opcode::I64Store32 => mem_op!(i64_store32),
            Opcode::MemorySize => single_argument!(MemIdx => memory_size),
            Opcode::MemoryGrow => single_argument!(MemIdx => memory_grow),
            Opcode::I32Const => {
                if let Some(constant) = input.get(..4) {
                    parser
                        .i32_const(i32::from_le_bytes(constant.try_into().unwrap()))
                        .to_parsed(start, kind)?;

                    &input[4..]
                } else {
                    return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                        input,
                        ErrorKind::Eof,
                        bad_argument(),
                    )));
                }
            }
            Opcode::I64Const => {
                if let Some(constant) = input.get(..8) {
                    parser
                        .i64_const(i64::from_le_bytes(constant.try_into().unwrap()))
                        .to_parsed(start, kind)?;

                    &input[8..]
                } else {
                    return Err(nom::Err::Failure(E::from_error_kind_and_cause(
                        input,
                        ErrorKind::Eof,
                        bad_argument(),
                    )));
                }
            }
            Opcode::F32Const => single_argument!(crate::values::F32 => f32_const),
            Opcode::F64Const => single_argument!(crate::values::F64 => f64_const),
            Opcode::I32Eqz => empty_case!(i32_eqz),
            Opcode::I32Eq => empty_case!(i32_eq),
            Opcode::I32Ne => empty_case!(i32_ne),
            Opcode::I32LtS => empty_case!(i32_lt_s),
            Opcode::I32LtU => empty_case!(i32_lt_u),
            Opcode::I32GtS => empty_case!(i32_gt_s),
            Opcode::I32GtU => empty_case!(i32_gt_u),
            Opcode::I32LeS => empty_case!(i32_le_s),
            Opcode::I32LeU => empty_case!(i32_le_u),
            Opcode::I32GeS => empty_case!(i32_lg_s),
            Opcode::I32GeU => empty_case!(i32_ge_u),
            Opcode::I64Eqz => empty_case!(i64_eqz),
            Opcode::I64Eq => empty_case!(i64_eq),
            Opcode::I64Ne => empty_case!(i64_ne),
            Opcode::I64LtS => empty_case!(i64_lt_s),
            Opcode::I64LtU => empty_case!(i64_lt_u),
            Opcode::I64GtS => empty_case!(i64_gt_s),
            Opcode::I64GtU => empty_case!(i64_gt_u),
            Opcode::I64LeS => empty_case!(i64_le_s),
            Opcode::I64LeU => empty_case!(i64_le_u),
            Opcode::I64GeS => empty_case!(i64_ge_s),
            Opcode::I64GeU => empty_case!(i64_ge_u),
            Opcode::F32Eq => empty_case!(f32_eq),
            Opcode::F32Ne => empty_case!(f32_ne),
            Opcode::F32Lt => empty_case!(f32_lt),
            Opcode::F32Gt => empty_case!(f32_gt),
            Opcode::F32Le => empty_case!(f32_le),
            Opcode::F32Ge => empty_case!(f32_ge),
            Opcode::F64Eq => empty_case!(f64_eq),
            Opcode::F64Ne => empty_case!(f64_ne),
            Opcode::F64Lt => empty_case!(f64_lt),
            Opcode::F64Gt => empty_case!(f64_gt),
            Opcode::F64Le => empty_case!(f64_le),
            Opcode::F64Ge => empty_case!(f64_ge),
            Opcode::I32Clz => empty_case!(i32_clz),
            Opcode::I32Ctz => empty_case!(i32_ctz),
            Opcode::I32Popcnt => empty_case!(i32_popcnt),
            Opcode::I32Add => empty_case!(i32_add),
            Opcode::I32Sub => empty_case!(i32_sub),
            Opcode::I32Mul => empty_case!(i32_mul),
            Opcode::I32DivS => empty_case!(i32_div_s),
            Opcode::I32DivU => empty_case!(i32_div_u),
            Opcode::I32RemS => empty_case!(i32_rem_s),
            Opcode::I32RemU => empty_case!(i32_rem_u),
            Opcode::I32And => empty_case!(i32_and),
            Opcode::I32Or => empty_case!(i32_or),
            Opcode::I32Xor => empty_case!(i32_xor),
            Opcode::I32Shl => empty_case!(i32_shl),
            Opcode::I32ShrS => empty_case!(i32_shr_s),
            Opcode::I32ShrU => empty_case!(i32_shr_u),
            Opcode::I32Rotl => empty_case!(i32_rotl),
            Opcode::I32Rotr => empty_case!(i32_rotr),
            Opcode::I64Clz => empty_case!(i64_clz),
            Opcode::I64Ctz => empty_case!(i64_ctz),
            Opcode::I64Popcnt => empty_case!(i64_popcnt),
            Opcode::I64Add => empty_case!(i64_add),
            Opcode::I64Sub => empty_case!(i64_sub),
            Opcode::I64Mul => empty_case!(i64_mul),
            Opcode::I64DivS => empty_case!(i64_div_s),
            Opcode::I64DivU => empty_case!(i64_div_u),
            Opcode::I64RemS => empty_case!(i64_rem_s),
            Opcode::I64RemU => empty_case!(i64_rem_u),
            Opcode::I64And => empty_case!(i64_and),
            Opcode::I64Or => empty_case!(i64_or),
            Opcode::I64Xor => empty_case!(i64_xor),
            Opcode::I64Shl => empty_case!(i64_shl),
            Opcode::I64ShrS => empty_case!(i64_shr_s),
            Opcode::I64ShrU => empty_case!(i64_shr_u),
            Opcode::I64Rotl => empty_case!(i64_rotl),
            Opcode::I64Rotr => empty_case!(i64_rotr),
            Opcode::F32Abs => empty_case!(f32_abs),
            Opcode::F32Neg => empty_case!(f32_neg),
            Opcode::F32Ceil => empty_case!(f32_ceil),
            Opcode::F32Floor => empty_case!(f32_floor),
            Opcode::F32Trunc => empty_case!(f32_trunc),
            Opcode::F32Nearest => empty_case!(f32_nearest),
            Opcode::F32Sqrt => empty_case!(f32_sqrt),
            Opcode::F32Add => empty_case!(f32_add),
            Opcode::F32Sub => empty_case!(f32_sub),
            Opcode::F32Mul => empty_case!(f32_mul),
            Opcode::F32Div => empty_case!(f32_div),
            Opcode::F32Min => empty_case!(f32_min),
            Opcode::F32Max => empty_case!(f32_max),
            Opcode::F32Copysign => empty_case!(f32_copysign),
            Opcode::F64Abs => empty_case!(f64_abs),
            Opcode::F64Neg => empty_case!(f64_neg),
            Opcode::F64Ceil => empty_case!(f64_ceil),
            Opcode::F64Floor => empty_case!(f64_floor),
            Opcode::F64Trunc => empty_case!(f64_trunc),
            Opcode::F64Nearest => empty_case!(f64_nearest),
            Opcode::F64Sqrt => empty_case!(f64_sqrt),
            Opcode::F64Add => empty_case!(f64_add),
            Opcode::F64Sub => empty_case!(f64_sub),
            Opcode::F64Mul => empty_case!(f64_mul),
            Opcode::F64Div => empty_case!(f64_div),
            Opcode::F64Min => empty_case!(f64_min),
            Opcode::F64Max => empty_case!(f64_max),
            Opcode::F64Copysign => empty_case!(f64_copysign),
            Opcode::I32WrapI64 => empty_case!(i32_wrap_i64),
            Opcode::I32TruncF32S => empty_case!(i32_trunc_f32_s),
            Opcode::I32TruncF32U => empty_case!(i32_trunc_f32_u),
            Opcode::I32TruncF64S => empty_case!(i32_trunc_f64_s),
            Opcode::I32TruncF64U => empty_case!(i32_trunc_f64_u),
            Opcode::I64ExtendI32S => empty_case!(i64_extend_i32_s),
            Opcode::I64ExtendI32U => empty_case!(i64_extend_i32_u),
            Opcode::I64TruncF32S => empty_case!(i64_trunc_f32_s),
            Opcode::I64TruncF32U => empty_case!(i64_trunc_f32_u),
            Opcode::I64TruncF64S => empty_case!(i64_trunc_f64_s),
            Opcode::I64TruncF64U => empty_case!(i64_trunc_f64_u),
            Opcode::F32ConvertI32S => empty_case!(f32_convert_i32_s),
            Opcode::F32ConvertI32U => empty_case!(f32_convert_i32_u),
            Opcode::F32ConvertI64S => empty_case!(f32_convert_i64_s),
            Opcode::F32ConvertI64U => empty_case!(f32_convert_i64_u),
            Opcode::F32DemoteF64 => empty_case!(f32_demote_f64),
            Opcode::F64ConvertI32S => empty_case!(f64_convert_i32_s),
            Opcode::F64ConvertI32U => empty_case!(f64_convert_i32_u),
            Opcode::F64ConvertI64S => empty_case!(f64_convert_i64_s),
            Opcode::F64ConvertI64U => empty_case!(f64_convert_i64_u),
            Opcode::F64PromoteF32 => empty_case!(f64_promote_f32),
            Opcode::I32ReinterpretF32 => empty_case!(i32_reinterpret_f32),
            Opcode::I64ReinterpretF64 => empty_case!(i64_reinterpret_f64),
            Opcode::F32ReinterpretI32 => empty_case!(f32_reinterpret_i32),
            Opcode::F64ReinterpretI64 => empty_case!(f64_reinterpret_i64),
            Opcode::I32Extend8S => empty_case!(i32_extend8_s),
            Opcode::I32Extend16S => empty_case!(i32_extend16_s),
            Opcode::I64Extend8S => empty_case!(i64_extend8_s),
            Opcode::I64Extend16S => empty_case!(i64_extend16_s),
            Opcode::I64Extend32S => empty_case!(i64_extend32_s),
            Opcode::RefNull => single_argument!(crate::types::RefType => ref_null),
            Opcode::RefIsNull => empty_case!(ref_is_null),
            Opcode::RefFunc => single_argument!(module::FuncIdx => ref_func),
            Opcode::TableGet => single_argument!(TableIdx => table_get),
            Opcode::TableSet => single_argument!(TableIdx => table_set),
            Opcode::ReturnCall => single_argument!(module::FuncIdx => return_call),
            Opcode::ReturnCallIndirect => {
                simple_arguments!(signature: TypeIdx, table: TableIdx => return_call_indirect)
            }
            Opcode::Try => block_start!(r#try),
            Opcode::Catch => single_argument!(module::TagIdx => r#catch),
            Opcode::Throw => single_argument!(module::TagIdx => r#throw),
            Opcode::Rethrow => single_argument!(isa::LabelIdx => rethrow),
            Opcode::Delegate => single_argument!(isa::LabelIdx => delegate),
            Opcode::CatchAll => empty_case!(catch_all),
        },
        InstrKind::FCPrefixed(opcode) => match opcode {
            FCPrefixedOpcode::I32TruncSatF32S => empty_case!(i32_trunc_sat_f32_s),
            FCPrefixedOpcode::I32TruncSatF32U => empty_case!(i32_trunc_sat_f32_u),
            FCPrefixedOpcode::I32TruncSatF64S => empty_case!(i32_trunc_sat_f64_s),
            FCPrefixedOpcode::I32TruncSatF64U => empty_case!(i32_trunc_sat_f64_u),
            FCPrefixedOpcode::I64TruncSatF32S => empty_case!(i64_trunc_sat_f32_s),
            FCPrefixedOpcode::I64TruncSatF32U => empty_case!(i64_trunc_sat_f32_u),
            FCPrefixedOpcode::I64TruncSatF64S => empty_case!(i64_trunc_sat_f64_s),
            FCPrefixedOpcode::I64TruncSatF64U => empty_case!(i64_trunc_sat_f64_u),
            FCPrefixedOpcode::MemoryCopy => copy_op!(MemIdx => memory_copy),
            FCPrefixedOpcode::MemoryFill => single_argument!(MemIdx => memory_fill),
            FCPrefixedOpcode::MemoryInit => {
                simple_arguments!(segment: module::DataIdx, memory: MemIdx => memory_init)
            }
            FCPrefixedOpcode::DataDrop => single_argument!(module::DataIdx => data_drop),
            FCPrefixedOpcode::TableCopy => copy_op!(TableIdx => table_copy),
            FCPrefixedOpcode::TableInit => {
                simple_arguments!(segment: module::ElemIdx, memory: TableIdx => table_init)
            }
            FCPrefixedOpcode::ElemDrop => single_argument!(module::ElemIdx => elem_drop),
            FCPrefixedOpcode::TableSize => single_argument!(TableIdx => table_size),
            FCPrefixedOpcode::TableGrow => single_argument!(TableIdx => table_grow),
            FCPrefixedOpcode::TableFill => single_argument!(TableIdx => table_fill),
        },
        InstrKind::V128(opcode) => match opcode {
            V128Opcode::V128Load => mem_op!(v128_load),
            V128Opcode::V128Load8x8S => mem_op!(v128_load8x8_s),
            V128Opcode::V128Load8x8U => mem_op!(v128_load8x8_u),
            V128Opcode::V128Load16x4S => mem_op!(v128_load16x4_s),
            V128Opcode::V128Load16x4U => mem_op!(v128_load16x4_u),
            V128Opcode::V128Load32x2S => mem_op!(v128_load32x2_s),
            V128Opcode::V128Load32x2U => mem_op!(v128_load32x2_u),
            V128Opcode::V128Load8Splat => mem_op!(v128_load8_splat),
            V128Opcode::V128Load16Splat => mem_op!(v128_load16_splat),
            V128Opcode::V128Load32Splat => mem_op!(v128_load32_splat),
            V128Opcode::V128Load64Splat => mem_op!(v128_load64_splat),
            V128Opcode::V128Load32Zero => mem_op!(v128_load32_zero),
            V128Opcode::V128Load64Zero => mem_op!(v128_load64_zero),
            V128Opcode::V128Store => mem_op!(v128_store),
            V128Opcode::V128Load8Lane => v128_mem_lane_op!(v128_load8_lane),
            V128Opcode::V128Load16Lane => v128_mem_lane_op!(v128_load16_lane),
            V128Opcode::V128Load32Lane => v128_mem_lane_op!(v128_load32_lane),
            V128Opcode::V128Load64Lane => v128_mem_lane_op!(v128_load64_lane),
            V128Opcode::V128Store8Lane => v128_mem_lane_op!(v128_store8_lane),
            V128Opcode::V128Store16Lane => v128_mem_lane_op!(v128_store16_lane),
            V128Opcode::V128Store32Lane => v128_mem_lane_op!(v128_store32_lane),
            V128Opcode::V128Store64Lane => v128_mem_lane_op!(v128_store64_lane),
            V128Opcode::V128Const => single_argument!(crate::values::V128 => v128_const),
            V128Opcode::I8x16Shuffle => {
                single_argument!(crate::values::V128ShuffleLanes => i8x16_shuffle)
            }
            V128Opcode::I8x16Swizzle => empty_case!(i8x16_swizzle),
            V128Opcode::I8x16Splat => empty_case!(i8x16_splat),
            V128Opcode::I16x8Splat => empty_case!(i16x8_splat),
            V128Opcode::I32x4Splat => empty_case!(i32x4_splat),
            V128Opcode::I64x2Splat => empty_case!(i64x2_splat),
            V128Opcode::F32x4Splat => empty_case!(f32x4_splat),
            V128Opcode::F64x2Splat => empty_case!(f64x2_splat),
            V128Opcode::I8x16ExtractLaneS => v128_lane_op!(i8x16_extract_lane_s),
            V128Opcode::I8x16ExtractLaneU => v128_lane_op!(i8x16_extract_lane_u),
            V128Opcode::I8x16ReplaceLane => v128_lane_op!(i8x16_replace_lane),
            V128Opcode::I16x8ExtractLaneS => v128_lane_op!(i16x8_extract_lane_s),
            V128Opcode::I16x8ExtractLaneU => v128_lane_op!(i16x8_extract_lane_u),
            V128Opcode::I16x8ReplaceLane => v128_lane_op!(i16x8_replace_lane),
            V128Opcode::I32x4ExtractLane => v128_lane_op!(i32x4_extract_lane),
            V128Opcode::I32x4ReplaceLane => v128_lane_op!(i32x4_replace_lane),
            V128Opcode::I64x2ExtractLane => v128_lane_op!(i64x2_extract_lane),
            V128Opcode::I64x2ReplaceLane => v128_lane_op!(i64x2_replace_lane),
            V128Opcode::F32x4ExtractLane => v128_lane_op!(f32x4_extract_lane),
            V128Opcode::F32x4ReplaceLane => v128_lane_op!(f32x4_replace_lane),
            V128Opcode::F64x2ExtractLane => v128_lane_op!(f64x4_extract_lane),
            V128Opcode::F64x2ReplaceLane => v128_lane_op!(f64x4_replace_lane),
            V128Opcode::I8x16Eq => empty_case!(i8x16_eq),
            V128Opcode::I8x16Ne => empty_case!(i8x16_ne),
            V128Opcode::I8x16LtS => empty_case!(i8x16_lt_s),
            V128Opcode::I8x16LtU => empty_case!(i8x16_lt_u),
            V128Opcode::I8x16GtS => empty_case!(i8x16_gt_s),
            V128Opcode::I8x16GtU => empty_case!(i8x16_gt_u),
            V128Opcode::I8x16LeS => empty_case!(i8x16_le_s),
            V128Opcode::I8x16LeU => empty_case!(i8x16_le_u),
            V128Opcode::I8x16GeS => empty_case!(i8x16_ge_s),
            V128Opcode::I8x16GeU => empty_case!(i8x16_ge_u),
            V128Opcode::I16x8Eq => empty_case!(i16x8_eq),
            V128Opcode::I16x8Ne => empty_case!(i16x8_ne),
            V128Opcode::I16x8LtS => empty_case!(i16x8_lt_s),
            V128Opcode::I16x8LtU => empty_case!(i16x8_lt_u),
            V128Opcode::I16x8GtS => empty_case!(i16x8_gt_s),
            V128Opcode::I16x8GtU => empty_case!(i16x8_gt_u),
            V128Opcode::I16x8LeS => empty_case!(i16x8_le_s),
            V128Opcode::I16x8LeU => empty_case!(i16x8_le_u),
            V128Opcode::I16x8GeS => empty_case!(i16x8_ge_s),
            V128Opcode::I16x8GeU => empty_case!(i16x8_ge_u),
            V128Opcode::I32x4Eq => empty_case!(i32x4_eq),
            V128Opcode::I32x4Ne => empty_case!(i32x4_ne),
            V128Opcode::I32x4LtS => empty_case!(i32x4_lt_s),
            V128Opcode::I32x4LtU => empty_case!(i32x4_lt_u),
            V128Opcode::I32x4GtS => empty_case!(i32x4_gt_s),
            V128Opcode::I32x4GtU => empty_case!(i32x4_gt_u),
            V128Opcode::I32x4LeS => empty_case!(i32x4_le_s),
            V128Opcode::I32x4LeU => empty_case!(i32x4_le_u),
            V128Opcode::I32x4GeS => empty_case!(i32x4_ge_s),
            V128Opcode::I32x4GeU => empty_case!(i32x4_ge_u),
            V128Opcode::F32x4Eq => empty_case!(f32x4_eq),
            V128Opcode::F32x4Ne => empty_case!(f32x4_ne),
            V128Opcode::F32x4Lt => empty_case!(f32x4_lt),
            V128Opcode::F32x4Gt => empty_case!(f32x4_gt),
            V128Opcode::F32x4Le => empty_case!(f32x4_le),
            V128Opcode::F32x4Ge => empty_case!(f32x4_ge),
            V128Opcode::F64x2Eq => empty_case!(f64x2_eq),
            V128Opcode::F64x2Ne => empty_case!(f64x2_ne),
            V128Opcode::F64x2Lt => empty_case!(f64x2_lt),
            V128Opcode::F64x2Gt => empty_case!(f64x2_gt),
            V128Opcode::F64x2Le => empty_case!(f64x2_le),
            V128Opcode::F64x2Ge => empty_case!(f64x2_ge),
            V128Opcode::V128Not => empty_case!(v128_not),
            V128Opcode::V128And => empty_case!(v128_and),
            V128Opcode::V128AndNot => empty_case!(v128_andnot),
            V128Opcode::V128Or => empty_case!(v128_or),
            V128Opcode::V128Xor => empty_case!(v128_xor),
            V128Opcode::V128Bitselect => empty_case!(v128_bitselect),
            V128Opcode::V128AnyTrue => empty_case!(v128_any_true),
            V128Opcode::F32x4DemoteF64x2Zero => empty_case!(f32x4_demote_f64x2_zero),
            V128Opcode::F64x2PromoteLowF32x4 => empty_case!(f64x2_promote_low_f32x4),
            V128Opcode::I8x16Abs => empty_case!(i8x16_abs),
            V128Opcode::I8x16Neg => empty_case!(i8x16_neg),
            V128Opcode::I8x16Popcnt => empty_case!(i8x16_popcnt),
            V128Opcode::I8x16AllTrue => empty_case!(i8x16_all_true),
            V128Opcode::I8x16Bitmask => empty_case!(i8x16_bitmask),
            V128Opcode::I8x16NarrowI16x8S => empty_case!(i8x16_narrow_i16x8_s),
            V128Opcode::I8x16NarrowI16x8U => empty_case!(i8x16_narrow_i16x8_u),
            V128Opcode::F32x4Ceil => empty_case!(f32x4_ceil),
            V128Opcode::F32x4Floor => empty_case!(f32x4_floor),
            V128Opcode::F32x4Trunc => empty_case!(f32x4_trunc),
            V128Opcode::F32x4Nearest => empty_case!(f32x4_nearest),
            V128Opcode::I8x16Shl => empty_case!(i8x16_shl),
            V128Opcode::I8x16ShrS => empty_case!(i8x16_shr_s),
            V128Opcode::I8x16ShrU => empty_case!(i8x16_shr_u),
            V128Opcode::I8x16Add => empty_case!(i8x16_add),
            V128Opcode::I8x16AddSatS => empty_case!(i8x16_add_sat_s),
            V128Opcode::I8x16AddSatU => empty_case!(i8x16_add_sat_u),
            V128Opcode::I8x16Sub => empty_case!(i8x16_sub),
            V128Opcode::I8x16SubSatS => empty_case!(i8x16_sub_sat_s),
            V128Opcode::I8x16SubSatU => empty_case!(i8x16_sub_sat_u),
            V128Opcode::F64x2Ceil => empty_case!(f64x2_ceil),
            V128Opcode::F64x2Floor => empty_case!(f64x2_floor),
            V128Opcode::I8x16MinS => empty_case!(i8x16_min_s),
            V128Opcode::I8x16MinU => empty_case!(i8x16_min_u),
            V128Opcode::I8x16MaxS => empty_case!(i8x16_max_s),
            V128Opcode::I8x16MaxU => empty_case!(i8x16_max_u),
            V128Opcode::F64x2Trunc => empty_case!(f64x2_trunc),
            V128Opcode::I8x16AvgrU => empty_case!(i8x16_avgr_u),
            V128Opcode::I16x8ExtaddPairwiseI8x16S => empty_case!(i16x8_extadd_pairwise_i8x16_s),
            V128Opcode::I16x8ExtaddPairwiseI8x16U => empty_case!(i16x8_extadd_pairwise_i8x16_u),
            V128Opcode::I32x4ExtaddPairwiseI16x8S => empty_case!(i32x4_extadd_pairwise_i16x8_s),
            V128Opcode::I32x4ExtaddPairwiseI16x8U => empty_case!(i32x4_extadd_pairwise_i16x8_u),
            V128Opcode::I16x8Abs => empty_case!(i16x8_abs),
            V128Opcode::I16x8Neg => empty_case!(i16x8_neg),
            V128Opcode::I16x8Q15mulrSatS => empty_case!(i16x8_q15mulr_sat_s),
            V128Opcode::I16x8AllTrue => empty_case!(i16x8_all_true),
            V128Opcode::I16x8Bitmask => empty_case!(i16x8_bitmask),
            V128Opcode::I16x8NarrowI32x4S => empty_case!(i16x8_narrow_i32x4_s),
            V128Opcode::I16x8NarrowI32x4U => empty_case!(i16x8_narrow_i32x4_u),
            V128Opcode::I16x8ExtendLowI8x16S => empty_case!(i16x8_extend_low_i8x16_s),
            V128Opcode::I16x8ExtendHighI8x16S => empty_case!(i16x8_extend_high_i8x16_s),
            V128Opcode::I16x8ExtendLowI8x16U => empty_case!(i16x8_extend_low_i8x16_u),
            V128Opcode::I16x8ExtendHighI8x16U => empty_case!(i16x8_extend_high_i8x16_u),
            V128Opcode::I16x8Shl => empty_case!(i16x8_shl),
            V128Opcode::I16x8ShrS => empty_case!(i16x8_shr_s),
            V128Opcode::I16x8ShrU => empty_case!(i16x8_shr_u),
            V128Opcode::I16x8Add => empty_case!(i16x8_add),
            V128Opcode::I16x8AddSatS => empty_case!(i16x8_add_sat_s),
            V128Opcode::I16x8AddSatU => empty_case!(i16x8_add_sat_u),
            V128Opcode::I16x8Sub => empty_case!(i16x8_sub),
            V128Opcode::I16x8SubSatS => empty_case!(i16x8_sub_sat_s),
            V128Opcode::I16x8SubSatU => empty_case!(i16x8_sub_sat_u),
            V128Opcode::F64x2Nearest => empty_case!(f64x2_nearest),
            V128Opcode::I16x8Mul => empty_case!(i16x8_mul),
            V128Opcode::I16x8MinS => empty_case!(i16x8_min_s),
            V128Opcode::I16x8MinU => empty_case!(i16x8_min_u),
            V128Opcode::I16x8MaxS => empty_case!(i16x8_max_s),
            V128Opcode::I16x8MaxU => empty_case!(i16x8_max_u),
            V128Opcode::I16x8AvgrU => empty_case!(i16x8_avgr_u),
            V128Opcode::I16x8ExtmulLowI8x16S => empty_case!(i16x8_extmul_low_i8x16_s),
            V128Opcode::I16x8ExtmulHighI8x16S => empty_case!(i16x8_extmul_high_i8x16_s),
            V128Opcode::I16x8ExtmulLowI8x16U => empty_case!(i16x8_extmul_low_i8x16_u),
            V128Opcode::I16x8ExtmulHighI8x16U => empty_case!(i16x8_extmul_high_i8x16_u),
            V128Opcode::I32x4Abs => empty_case!(i32x4_abs),
            V128Opcode::I32x4Neg => empty_case!(i32x4_neg),
            V128Opcode::I32x4AllTrue => empty_case!(i32x4_all_true),
            V128Opcode::I32x4Bitmask => empty_case!(i32x4_bitmask),
            V128Opcode::I32x4ExtendLowI16x8S => empty_case!(i32x4_extend_low_i16x8_s),
            V128Opcode::I32x4ExtendHighI16x8S => empty_case!(i32x4_extend_high_i16x8_s),
            V128Opcode::I32x4ExtendLowI16x8U => empty_case!(i32x4_extend_low_i16x8_u),
            V128Opcode::I32x4ExtendHighI16x8U => empty_case!(i32x4_extend_high_i16x8_u),
            V128Opcode::I32x4Shl => empty_case!(i32x4_shl),
            V128Opcode::I32x4ShrS => empty_case!(i32x4_shr_s),
            V128Opcode::I32x4ShrU => empty_case!(i32x4_shr_u),
            V128Opcode::I32x4Add => empty_case!(i32x4_add),
            V128Opcode::I32x4Sub => empty_case!(i32x4_sub),
            V128Opcode::I32x4Mul => empty_case!(i32x4_mul),
            V128Opcode::I32x4MinS => empty_case!(i32x4_min_s),
            V128Opcode::I32x4MinU => empty_case!(i32x4_min_u),
            V128Opcode::I32x4MaxS => empty_case!(i32x4_max_s),
            V128Opcode::I32x4MaxU => empty_case!(i32x4_max_u),
            V128Opcode::I32x4DotI16x8S => empty_case!(i32x4_dot_i16x8_s),
            V128Opcode::I32x4ExtmulLowI16x8S => empty_case!(i32x4_extmul_low_i16x8_s),
            V128Opcode::I32x4ExtmulHighI16x8S => empty_case!(i32x4_extmul_high_i16x8_s),
            V128Opcode::I32x4ExtmulLowI16x8U => empty_case!(i32x4_extmul_low_i16x8_u),
            V128Opcode::I32x4ExtmulHighI16x8U => empty_case!(i32x4_extmul_high_i16x8_u),
            V128Opcode::I64x2Abs => empty_case!(i64x2_abs),
            V128Opcode::I64x2Neg => empty_case!(i64x2_neg),
            V128Opcode::I64x2AllTrue => empty_case!(i64x2_all_true),
            V128Opcode::I64x2Bitmask => empty_case!(i64x2_bitmask),
            V128Opcode::I64x2ExtendLowI32x4S => empty_case!(i64x2_extend_low_i32x4_s),
            V128Opcode::I64x2ExtendHighI32x4S => empty_case!(i64x2_extend_high_i32x4_s),
            V128Opcode::I64x2ExtendLowI32x4U => empty_case!(i64x2_extend_low_i32x4_u),
            V128Opcode::I64x2ExtendHighI32x4U => empty_case!(i64x2_extend_high_i32x4_u),
            V128Opcode::I64x2Shl => empty_case!(i64x2_shl),
            V128Opcode::I64x2ShrS => empty_case!(i64x2_shr_s),
            V128Opcode::I64x2ShrU => empty_case!(i64x2_shr_u),
            V128Opcode::I64x2Add => empty_case!(i64x2_add),
            V128Opcode::I64x2Sub => empty_case!(i64x2_sub),
            V128Opcode::I64x2Mul => empty_case!(i64x2_mul),
            V128Opcode::I64x2Eq => empty_case!(i64x2_eq),
            V128Opcode::I64x2Ne => empty_case!(i64x2_ne),
            V128Opcode::I64x2LtS => empty_case!(i64x2_lt_s),
            V128Opcode::I64x2GtS => empty_case!(i64x2_gt_s),
            V128Opcode::I64x2LeS => empty_case!(i64x2_le_s),
            V128Opcode::I64x2GeS => empty_case!(i64x2_ge_s),
            V128Opcode::I64x2ExtmulLowI32x4S => empty_case!(i64x2_extmul_low_i32x4_s),
            V128Opcode::I64x2ExtmulHighI32x4S => empty_case!(i64x2_extmul_high_i32x4_s),
            V128Opcode::I64x2ExtmulLowI32x4U => empty_case!(i64x2_extmul_low_i32x4_u),
            V128Opcode::I64x2ExtmulHighI32x4U => empty_case!(i64x2_extmul_high_i32x4_u),
            V128Opcode::F32x4Abs => empty_case!(f32x4_abs),
            V128Opcode::F32x4Neg => empty_case!(f32x4_neg),
            V128Opcode::F32x4Sqrt => empty_case!(f32x4_sqrt),
            V128Opcode::F32x4Add => empty_case!(f32x4_add),
            V128Opcode::F32x4Sub => empty_case!(f32x4_sub),
            V128Opcode::F32x4Mul => empty_case!(f32x4_mul),
            V128Opcode::F32x4Div => empty_case!(f32x4_div),
            V128Opcode::F32x4Min => empty_case!(f32x4_min),
            V128Opcode::F32x4Max => empty_case!(f32x4_max),
            V128Opcode::F32x4Pmin => empty_case!(f32x4_pmin),
            V128Opcode::F32x4Pmax => empty_case!(f32x4_pmax),
            V128Opcode::F64x2Abs => empty_case!(f64x2_abs),
            V128Opcode::F64x2Neg => empty_case!(f64x2_neg),
            V128Opcode::F64x2Sqrt => empty_case!(f64x2_sqrt),
            V128Opcode::F64x2Add => empty_case!(f64x2_add),
            V128Opcode::F64x2Sub => empty_case!(f64x2_sub),
            V128Opcode::F64x2Mul => empty_case!(f64x2_mul),
            V128Opcode::F64x2Div => empty_case!(f64x2_div),
            V128Opcode::F64x2Min => empty_case!(f64x2_min),
            V128Opcode::F64x2Max => empty_case!(f64x2_max),
            V128Opcode::F64x2Pmin => empty_case!(f64x2_pmin),
            V128Opcode::F64x2Pmax => empty_case!(f64x2_pmax),
            V128Opcode::I32x4TruncSatF32x4S => empty_case!(i32x4_trunc_sat_f32x4_s),
            V128Opcode::I32x4TruncSatF32x4U => empty_case!(i32x4_trunc_sat_f32x4_u),
            V128Opcode::F32x4ConvertI32x4S => empty_case!(f32x4_convert_i32x4_s),
            V128Opcode::F32x4ConvertI32x4U => empty_case!(f32x4_convert_i32x4_u),
            V128Opcode::I32x4TruncSatF64x2SZero => empty_case!(i32x4_trunc_sat_f64x2_s_zero),
            V128Opcode::I32x4TruncSatF64x2UZero => empty_case!(i32x4_trunc_sat_f64x2_u_zero),
            V128Opcode::F64x2ConvertLowI32x4S => empty_case!(f64x2_convert_low_i32x4_s),
            V128Opcode::F64x2ConvertLowI32x4U => empty_case!(f64x2_convert_low_i32x4_u),
            V128Opcode::I8x16RelaxedSwizzle => empty_case!(i8x16_relaxed_swizzle),
            V128Opcode::I32x4RelaxedTruncF32x4S => empty_case!(i32x4_relaxed_trunc_f32x4_s),
            V128Opcode::I32x4RelaxedTruncF32x4U => empty_case!(i32x4_relaxed_trunc_f32x4_u),
            V128Opcode::I32x4RelaxedTruncF64x2SZero => {
                empty_case!(i32x4_relaxed_trunc_f64x2_s_zero)
            }
            V128Opcode::I32x4RelaxedTruncF64x2UZero => {
                empty_case!(i32x4_relaxed_trunc_f64x2_u_zero)
            }
            V128Opcode::F32x4RelaxedMadd => empty_case!(f32x4_relaxed_madd),
            V128Opcode::F32x4RelaxedNmadd => empty_case!(f32x4_relaxed_nmadd),
            V128Opcode::F64x2RelaxedMadd => empty_case!(f64x2_relaxed_madd),
            V128Opcode::F64x2RelaxedNmadd => empty_case!(f64x2_relaxed_nmadd),
            V128Opcode::I8x16RelaxedLaneselect => empty_case!(i8x16_relaxed_laneselect),
            V128Opcode::I16x8RelaxedLaneselect => empty_case!(i16x8_relaxed_laneselect),
            V128Opcode::I32x4RelaxedLaneselect => empty_case!(i32x4_relaxed_laneselect),
            V128Opcode::I64x2RelaxedLaneselect => empty_case!(i64x2_relaxed_laneselect),
            V128Opcode::F32x4RelaxedMin => empty_case!(f32x4_relaxed_min),
            V128Opcode::F32x4RelaxedMax => empty_case!(f32x4_relaxed_max),
            V128Opcode::F64x2RelaxedMin => empty_case!(f64x2_relaxed_min),
            V128Opcode::F64x2RelaxedMax => empty_case!(f64x2_relaxed_max),
            V128Opcode::I16x8RelaxedQ15mulrS => empty_case!(i16x8_relaxed_q15mulr_s),
            V128Opcode::I16x8RelaxedDotI8x16I7x16S => empty_case!(i16x8_relaxed_dot_i8x16_i7x16_s),
            V128Opcode::I32x4RelaxedDotI8x16I7x16AddS => {
                empty_case!(i32x4_relaxed_dot_i8x16_i7x16_add_s)
            }
        },
        InstrKind::FEPrefixed(opcode) => match opcode {
            FEPrefixedOpcode::MemoryAtomicNotify => mem_op!(memory_atomic_notify),
            FEPrefixedOpcode::MemoryAtomicWait32 => mem_op!(memory_atomic_wait32),
            FEPrefixedOpcode::MemoryAtomicWait64 => mem_op!(memory_atomic_wait64),
            FEPrefixedOpcode::I32AtomicLoad => mem_op!(i32_atomic_load),
            FEPrefixedOpcode::I64AtomicLoad => mem_op!(i64_atomic_load),
            FEPrefixedOpcode::I32AtomicLoad8U => mem_op!(i32_atomic_load8_u),
            FEPrefixedOpcode::I32AtomicLoad16U => mem_op!(i32_atomic_load16_u),
            FEPrefixedOpcode::I64AtomicLoad8U => mem_op!(i64_atomic_load8_u),
            FEPrefixedOpcode::I64AtomicLoad16U => mem_op!(i64_atomic_load16_u),
            FEPrefixedOpcode::I64AtomicLoad32U => mem_op!(i64_atomic_load32_u),
            FEPrefixedOpcode::I32AtomicStore => mem_op!(i32_atomic_store),
            FEPrefixedOpcode::I64AtomicStore => mem_op!(i64_atomic_store),
            FEPrefixedOpcode::I32AtomicStore8U => mem_op!(i32_atomic_store8_u),
            FEPrefixedOpcode::I32AtomicStore16U => mem_op!(i32_atomic_store16_u),
            FEPrefixedOpcode::I64AtomicStore8U => mem_op!(i64_atomic_store8_u),
            FEPrefixedOpcode::I64AtomicStore16U => mem_op!(i64_atomic_store16_u),
            FEPrefixedOpcode::I64AtomicStore32U => mem_op!(i64_atomic_store32_u),
            FEPrefixedOpcode::I32AtomicRmwAdd => mem_op!(i32_atomic_rmw_add),
            FEPrefixedOpcode::I64AtomicRmwAdd => mem_op!(i64_atomic_rmw_add),
            FEPrefixedOpcode::I32AtomicRmw8AddU => mem_op!(i32_atomic_rmw8_add_u),
            FEPrefixedOpcode::I32AtomicRmw16AddU => mem_op!(i32_atomic_rmw16_add_u),
            FEPrefixedOpcode::I64AtomicRmw8AddU => mem_op!(i64_atomic_rmw8_add_u),
            FEPrefixedOpcode::I64AtomicRmw16AddU => mem_op!(i64_atomic_rmw16_add_u),
            FEPrefixedOpcode::I64AtomicRmw32AddU => mem_op!(i64_atomic_rmw32_add_u),
            FEPrefixedOpcode::I32AtomicRmwSub => mem_op!(i32_atomic_rmw_sub),
            FEPrefixedOpcode::I64AtomicRmwSub => mem_op!(i64_atomic_rmw_sub),
            FEPrefixedOpcode::I32AtomicRmw8SubU => mem_op!(i32_atomic_rmw8_sub_u),
            FEPrefixedOpcode::I32AtomicRmw16SubU => mem_op!(i32_atomic_rmw16_sub_u),
            FEPrefixedOpcode::I64AtomicRmw8SubU => mem_op!(i64_atomic_rmw8_sub_u),
            FEPrefixedOpcode::I64AtomicRmw16SubU => mem_op!(i64_atomic_rmw16_sub_u),
            FEPrefixedOpcode::I64AtomicRmw32SubU => mem_op!(i64_atomic_rmw32_sub_u),
            FEPrefixedOpcode::I32AtomicRmwAnd => mem_op!(i32_atomic_rmw_and),
            FEPrefixedOpcode::I64AtomicRmwAnd => mem_op!(i64_atomic_rmw_and),
            FEPrefixedOpcode::I32AtomicRmw8AndU => mem_op!(i32_atomic_rmw8_and_u),
            FEPrefixedOpcode::I32AtomicRmw16AndU => mem_op!(i32_atomic_rmw16_and_u),
            FEPrefixedOpcode::I64AtomicRmw8AndU => mem_op!(i64_atomic_rmw8_and_u),
            FEPrefixedOpcode::I64AtomicRmw16AndU => mem_op!(i64_atomic_rmw16_and_u),
            FEPrefixedOpcode::I64AtomicRmw32AndU => mem_op!(i64_atomic_rmw32_and_u),
            FEPrefixedOpcode::I32AtomicRmwOr => mem_op!(i32_atomic_rmw_or),
            FEPrefixedOpcode::I64AtomicRmwOr => mem_op!(i64_atomic_rmw_or),
            FEPrefixedOpcode::I32AtomicRmw8OrU => mem_op!(i32_atomic_rmw8_or_u),
            FEPrefixedOpcode::I32AtomicRmw16OrU => mem_op!(i32_atomic_rmw16_or_u),
            FEPrefixedOpcode::I64AtomicRmw8OrU => mem_op!(i64_atomic_rmw8_or_u),
            FEPrefixedOpcode::I64AtomicRmw16OrU => mem_op!(i64_atomic_rmw16_or_u),
            FEPrefixedOpcode::I64AtomicRmw32OrU => mem_op!(i64_atomic_rmw32_or_u),
            FEPrefixedOpcode::I32AtomicRmwXor => mem_op!(i32_atomic_rmw_xor),
            FEPrefixedOpcode::I64AtomicRmwXor => mem_op!(i64_atomic_rmw_xor),
            FEPrefixedOpcode::I32AtomicRmw8XorU => mem_op!(i32_atomic_rmw8_xor_u),
            FEPrefixedOpcode::I32AtomicRmw16XorU => mem_op!(i32_atomic_rmw16_xor_u),
            FEPrefixedOpcode::I64AtomicRmw8XorU => mem_op!(i64_atomic_rmw8_xor_u),
            FEPrefixedOpcode::I64AtomicRmw16XorU => mem_op!(i64_atomic_rmw16_xor_u),
            FEPrefixedOpcode::I64AtomicRmw32XorU => mem_op!(i64_atomic_rmw32_xor_u),
            FEPrefixedOpcode::I32AtomicRmwXchg => mem_op!(i32_atomic_rmw_xchg),
            FEPrefixedOpcode::I64AtomicRmwXchg => mem_op!(i64_atomic_rmw_xchg),
            FEPrefixedOpcode::I32AtomicRmw8XchgU => mem_op!(i32_atomic_rmw8_xchg_u),
            FEPrefixedOpcode::I32AtomicRmw16XchgU => mem_op!(i32_atomic_rmw16_xchg_u),
            FEPrefixedOpcode::I64AtomicRmw8XchgU => mem_op!(i64_atomic_rmw8_xchg_u),
            FEPrefixedOpcode::I64AtomicRmw16XchgU => mem_op!(i64_atomic_rmw16_xchg_u),
            FEPrefixedOpcode::I64AtomicRmw32XchgU => mem_op!(i64_atomic_rmw32_xchg_u),
            FEPrefixedOpcode::I32AtomicRmwCmpxchg => mem_op!(i32_atomic_rmw_cmpxchg),
            FEPrefixedOpcode::I64AtomicRmwCmpxchg => mem_op!(i64_atomic_rmw_cmpxchg),
            FEPrefixedOpcode::I32AtomicRmw8CmpxchgU => mem_op!(i32_atomic_rmw8_cmpxchg_u),
            FEPrefixedOpcode::I32AtomicRmw16CmpxchgU => mem_op!(i32_atomic_rmw16_cmpxchg_u),
            FEPrefixedOpcode::I64AtomicRmw8CmpxchgU => mem_op!(i64_atomic_rmw8_cmpxchg_u),
            FEPrefixedOpcode::I64AtomicRmw16CmpxchgU => mem_op!(i64_atomic_rmw16_cmpxchg_u),
            FEPrefixedOpcode::I64AtomicRmw32CmpxchgU => mem_op!(i64_atomic_rmw32_cmpxchg_u),
        },
    };

    Ok((input, parser))
}
