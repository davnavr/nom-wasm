use crate::{
    error::{ErrorSource, InvalidInstr},
    isa::{Opcode, ParseInstr},
};

trait ResultExt<'a, T, E: ErrorSource<'a>> {
    fn to_parsed(self, input: &'a [u8], opcode: Opcode) -> crate::input::Result<T, E>;
}

impl<'a, T, E: ErrorSource<'a>> ResultExt<'a, T, E> for crate::isa::Result<T, E> {
    #[inline]
    fn to_parsed(self, start: &'a [u8], opcode: Opcode) -> crate::input::Result<T, E> {
        use crate::isa::ParseInstrError;

        match self {
            Ok(value) => Ok(value),
            Err(ParseInstrError::Unrecognized) => Err(nom::Err::Failure(E::from_error_cause(
                start,
                crate::error::ErrorCause::Instr {
                    opcode,
                    reason: InvalidInstr::Unrecognized,
                },
            ))),
            Err(ParseInstrError::Cause(cause)) => {
                Err(nom::Err::Failure(E::from_error_cause(start, cause)))
            }
            Err(ParseInstrError::Nom(err)) => Err(err),
        }
    }
}

fn parse<'a, E, P>(input: &'a [u8], mut parser: P) -> crate::Parsed<'a, P, E>
where
    E: ErrorSource<'a>,
    P: ParseInstr<'a, E>,
{
    use crate::{
        error::AddCause as _,
        index::Index as _,
        isa,
        module::{self, MemIdx, TableIdx, TypeIdx},
    };

    let start = &input;
    let (input, opcode) = Opcode::parse(input)?;

    let bad_instr = move |reason| crate::error::ErrorCause::Instr { opcode, reason };
    let bad_argument = move || bad_instr(InvalidInstr::Argument);

    let parse_lane_idx = move |input: &'a [u8]| -> crate::Parsed<'a, isa::LaneIdx, E> {
        if let Some((lane, input)) = input.split_first() {
            Ok((input, *lane))
        } else {
            Err(nom::Err::Failure(E::from_error_cause(
                input,
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
                let (input, $parameter) = <$argument>::parse(input)
                    .add_cause_with(move || (input, bad_argument()))?;
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
                .add_cause_with(move || (input, bad_instr(InvalidInstr::Destination)))?;

            let (input, source) = <$index>::parse(input)
                .add_cause_with(move || (input, bad_instr(InvalidInstr::Source)))?;

            parser.$case(destination, source).to_parsed(start, opcode)?;
            input
        }};
    }

    macro_rules! v128_mem_lane_op {
        ($case:ident) => {{
            let (input, memarg) =
                isa::MemArg::parse(input).add_cause_with(move || (input, bad_argument()))?;

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
            let mut targets = isa::BrTableTargets::with_input(input)
                .add_cause_with(move || (input, bad_argument()))?;

            parser.br_table(&mut targets).to_parsed(start, opcode)?;
            targets
                .finish()
                .add_cause_with(move || (input, bad_argument()))?
                .0
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
                .add_cause_with(move || (input, bad_argument()))?;

            parser.select_typed(&mut types).to_parsed(start, opcode)?;
            types
                .into_parser()
                .add_cause_with(move || (input, bad_argument()))?
                .0
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
            let (input, n) =
                crate::values::leb128_s32(input).add_cause_with(move || (input, bad_argument()))?;

            parser.i32_const(n).to_parsed(start, opcode)?;
            input
        }
        Opcode::I64Const => {
            let (input, n) =
                crate::values::leb128_s64(input).add_cause_with(move || (input, bad_argument()))?;

            parser.i64_const(n).to_parsed(start, opcode)?;
            input
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
        Opcode::I32TruncSatF32S => empty_case!(i32_trunc_sat_f32_s),
        Opcode::I32TruncSatF32U => empty_case!(i32_trunc_sat_f32_u),
        Opcode::I32TruncSatF64S => empty_case!(i32_trunc_sat_f64_s),
        Opcode::I32TruncSatF64U => empty_case!(i32_trunc_sat_f64_u),
        Opcode::I64TruncSatF32S => empty_case!(i64_trunc_sat_f32_s),
        Opcode::I64TruncSatF32U => empty_case!(i64_trunc_sat_f32_u),
        Opcode::I64TruncSatF64S => empty_case!(i64_trunc_sat_f64_s),
        Opcode::I64TruncSatF64U => empty_case!(i64_trunc_sat_f64_u),
        Opcode::MemoryCopy => copy_op!(MemIdx => memory_copy),
        Opcode::MemoryFill => single_argument!(MemIdx => memory_fill),
        Opcode::MemoryInit => {
            simple_arguments!(segment: module::DataIdx, memory: MemIdx => memory_init)
        }
        Opcode::DataDrop => single_argument!(module::DataIdx => data_drop),
        Opcode::TableCopy => copy_op!(TableIdx => table_copy),
        Opcode::TableInit => {
            simple_arguments!(segment: module::ElemIdx, memory: TableIdx => table_init)
        }
        Opcode::ElemDrop => single_argument!(module::ElemIdx => elem_drop),
        Opcode::TableSize => single_argument!(TableIdx => table_size),
        Opcode::TableGrow => single_argument!(TableIdx => table_grow),
        Opcode::TableFill => single_argument!(TableIdx => table_fill),
        Opcode::V128Load => mem_op!(v128_load),
        Opcode::V128Load8x8S => mem_op!(v128_load8x8_s),
        Opcode::V128Load8x8U => mem_op!(v128_load8x8_u),
        Opcode::V128Load16x4S => mem_op!(v128_load16x4_s),
        Opcode::V128Load16x4U => mem_op!(v128_load16x4_u),
        Opcode::V128Load32x2S => mem_op!(v128_load32x2_s),
        Opcode::V128Load32x2U => mem_op!(v128_load32x2_u),
        Opcode::V128Load8Splat => mem_op!(v128_load8_splat),
        Opcode::V128Load16Splat => mem_op!(v128_load16_splat),
        Opcode::V128Load32Splat => mem_op!(v128_load32_splat),
        Opcode::V128Load64Splat => mem_op!(v128_load64_splat),
        Opcode::V128Load32Zero => mem_op!(v128_load32_zero),
        Opcode::V128Load64Zero => mem_op!(v128_load64_zero),
        Opcode::V128Store => mem_op!(v128_store),
        Opcode::V128Load8Lane => v128_mem_lane_op!(v128_load8_lane),
        Opcode::V128Load16Lane => v128_mem_lane_op!(v128_load16_lane),
        Opcode::V128Load32Lane => v128_mem_lane_op!(v128_load32_lane),
        Opcode::V128Load64Lane => v128_mem_lane_op!(v128_load64_lane),
        Opcode::V128Store8Lane => v128_mem_lane_op!(v128_store8_lane),
        Opcode::V128Store16Lane => v128_mem_lane_op!(v128_store16_lane),
        Opcode::V128Store32Lane => v128_mem_lane_op!(v128_store32_lane),
        Opcode::V128Store64Lane => v128_mem_lane_op!(v128_store64_lane),
        Opcode::V128Const => single_argument!(crate::values::V128 => v128_const),
        Opcode::I8x16Shuffle => {
            single_argument!(crate::values::V128ShuffleLanes => i8x16_shuffle)
        }
        Opcode::I8x16Swizzle => empty_case!(i8x16_swizzle),
        Opcode::I8x16Splat => empty_case!(i8x16_splat),
        Opcode::I16x8Splat => empty_case!(i16x8_splat),
        Opcode::I32x4Splat => empty_case!(i32x4_splat),
        Opcode::I64x2Splat => empty_case!(i64x2_splat),
        Opcode::F32x4Splat => empty_case!(f32x4_splat),
        Opcode::F64x2Splat => empty_case!(f64x2_splat),
        Opcode::I8x16ExtractLaneS => v128_lane_op!(i8x16_extract_lane_s),
        Opcode::I8x16ExtractLaneU => v128_lane_op!(i8x16_extract_lane_u),
        Opcode::I8x16ReplaceLane => v128_lane_op!(i8x16_replace_lane),
        Opcode::I16x8ExtractLaneS => v128_lane_op!(i16x8_extract_lane_s),
        Opcode::I16x8ExtractLaneU => v128_lane_op!(i16x8_extract_lane_u),
        Opcode::I16x8ReplaceLane => v128_lane_op!(i16x8_replace_lane),
        Opcode::I32x4ExtractLane => v128_lane_op!(i32x4_extract_lane),
        Opcode::I32x4ReplaceLane => v128_lane_op!(i32x4_replace_lane),
        Opcode::I64x2ExtractLane => v128_lane_op!(i64x2_extract_lane),
        Opcode::I64x2ReplaceLane => v128_lane_op!(i64x2_replace_lane),
        Opcode::F32x4ExtractLane => v128_lane_op!(f32x4_extract_lane),
        Opcode::F32x4ReplaceLane => v128_lane_op!(f32x4_replace_lane),
        Opcode::F64x2ExtractLane => v128_lane_op!(f64x4_extract_lane),
        Opcode::F64x2ReplaceLane => v128_lane_op!(f64x4_replace_lane),
        Opcode::I8x16Eq => empty_case!(i8x16_eq),
        Opcode::I8x16Ne => empty_case!(i8x16_ne),
        Opcode::I8x16LtS => empty_case!(i8x16_lt_s),
        Opcode::I8x16LtU => empty_case!(i8x16_lt_u),
        Opcode::I8x16GtS => empty_case!(i8x16_gt_s),
        Opcode::I8x16GtU => empty_case!(i8x16_gt_u),
        Opcode::I8x16LeS => empty_case!(i8x16_le_s),
        Opcode::I8x16LeU => empty_case!(i8x16_le_u),
        Opcode::I8x16GeS => empty_case!(i8x16_ge_s),
        Opcode::I8x16GeU => empty_case!(i8x16_ge_u),
        Opcode::I16x8Eq => empty_case!(i16x8_eq),
        Opcode::I16x8Ne => empty_case!(i16x8_ne),
        Opcode::I16x8LtS => empty_case!(i16x8_lt_s),
        Opcode::I16x8LtU => empty_case!(i16x8_lt_u),
        Opcode::I16x8GtS => empty_case!(i16x8_gt_s),
        Opcode::I16x8GtU => empty_case!(i16x8_gt_u),
        Opcode::I16x8LeS => empty_case!(i16x8_le_s),
        Opcode::I16x8LeU => empty_case!(i16x8_le_u),
        Opcode::I16x8GeS => empty_case!(i16x8_ge_s),
        Opcode::I16x8GeU => empty_case!(i16x8_ge_u),
        Opcode::I32x4Eq => empty_case!(i32x4_eq),
        Opcode::I32x4Ne => empty_case!(i32x4_ne),
        Opcode::I32x4LtS => empty_case!(i32x4_lt_s),
        Opcode::I32x4LtU => empty_case!(i32x4_lt_u),
        Opcode::I32x4GtS => empty_case!(i32x4_gt_s),
        Opcode::I32x4GtU => empty_case!(i32x4_gt_u),
        Opcode::I32x4LeS => empty_case!(i32x4_le_s),
        Opcode::I32x4LeU => empty_case!(i32x4_le_u),
        Opcode::I32x4GeS => empty_case!(i32x4_ge_s),
        Opcode::I32x4GeU => empty_case!(i32x4_ge_u),
        Opcode::F32x4Eq => empty_case!(f32x4_eq),
        Opcode::F32x4Ne => empty_case!(f32x4_ne),
        Opcode::F32x4Lt => empty_case!(f32x4_lt),
        Opcode::F32x4Gt => empty_case!(f32x4_gt),
        Opcode::F32x4Le => empty_case!(f32x4_le),
        Opcode::F32x4Ge => empty_case!(f32x4_ge),
        Opcode::F64x2Eq => empty_case!(f64x2_eq),
        Opcode::F64x2Ne => empty_case!(f64x2_ne),
        Opcode::F64x2Lt => empty_case!(f64x2_lt),
        Opcode::F64x2Gt => empty_case!(f64x2_gt),
        Opcode::F64x2Le => empty_case!(f64x2_le),
        Opcode::F64x2Ge => empty_case!(f64x2_ge),
        Opcode::V128Not => empty_case!(v128_not),
        Opcode::V128And => empty_case!(v128_and),
        Opcode::V128AndNot => empty_case!(v128_andnot),
        Opcode::V128Or => empty_case!(v128_or),
        Opcode::V128Xor => empty_case!(v128_xor),
        Opcode::V128Bitselect => empty_case!(v128_bitselect),
        Opcode::V128AnyTrue => empty_case!(v128_any_true),
        Opcode::F32x4DemoteF64x2Zero => empty_case!(f32x4_demote_f64x2_zero),
        Opcode::F64x2PromoteLowF32x4 => empty_case!(f64x2_promote_low_f32x4),
        Opcode::I8x16Abs => empty_case!(i8x16_abs),
        Opcode::I8x16Neg => empty_case!(i8x16_neg),
        Opcode::I8x16Popcnt => empty_case!(i8x16_popcnt),
        Opcode::I8x16AllTrue => empty_case!(i8x16_all_true),
        Opcode::I8x16Bitmask => empty_case!(i8x16_bitmask),
        Opcode::I8x16NarrowI16x8S => empty_case!(i8x16_narrow_i16x8_s),
        Opcode::I8x16NarrowI16x8U => empty_case!(i8x16_narrow_i16x8_u),
        Opcode::F32x4Ceil => empty_case!(f32x4_ceil),
        Opcode::F32x4Floor => empty_case!(f32x4_floor),
        Opcode::F32x4Trunc => empty_case!(f32x4_trunc),
        Opcode::F32x4Nearest => empty_case!(f32x4_nearest),
        Opcode::I8x16Shl => empty_case!(i8x16_shl),
        Opcode::I8x16ShrS => empty_case!(i8x16_shr_s),
        Opcode::I8x16ShrU => empty_case!(i8x16_shr_u),
        Opcode::I8x16Add => empty_case!(i8x16_add),
        Opcode::I8x16AddSatS => empty_case!(i8x16_add_sat_s),
        Opcode::I8x16AddSatU => empty_case!(i8x16_add_sat_u),
        Opcode::I8x16Sub => empty_case!(i8x16_sub),
        Opcode::I8x16SubSatS => empty_case!(i8x16_sub_sat_s),
        Opcode::I8x16SubSatU => empty_case!(i8x16_sub_sat_u),
        Opcode::F64x2Ceil => empty_case!(f64x2_ceil),
        Opcode::F64x2Floor => empty_case!(f64x2_floor),
        Opcode::I8x16MinS => empty_case!(i8x16_min_s),
        Opcode::I8x16MinU => empty_case!(i8x16_min_u),
        Opcode::I8x16MaxS => empty_case!(i8x16_max_s),
        Opcode::I8x16MaxU => empty_case!(i8x16_max_u),
        Opcode::F64x2Trunc => empty_case!(f64x2_trunc),
        Opcode::I8x16AvgrU => empty_case!(i8x16_avgr_u),
        Opcode::I16x8ExtaddPairwiseI8x16S => empty_case!(i16x8_extadd_pairwise_i8x16_s),
        Opcode::I16x8ExtaddPairwiseI8x16U => empty_case!(i16x8_extadd_pairwise_i8x16_u),
        Opcode::I32x4ExtaddPairwiseI16x8S => empty_case!(i32x4_extadd_pairwise_i16x8_s),
        Opcode::I32x4ExtaddPairwiseI16x8U => empty_case!(i32x4_extadd_pairwise_i16x8_u),
        Opcode::I16x8Abs => empty_case!(i16x8_abs),
        Opcode::I16x8Neg => empty_case!(i16x8_neg),
        Opcode::I16x8Q15mulrSatS => empty_case!(i16x8_q15mulr_sat_s),
        Opcode::I16x8AllTrue => empty_case!(i16x8_all_true),
        Opcode::I16x8Bitmask => empty_case!(i16x8_bitmask),
        Opcode::I16x8NarrowI32x4S => empty_case!(i16x8_narrow_i32x4_s),
        Opcode::I16x8NarrowI32x4U => empty_case!(i16x8_narrow_i32x4_u),
        Opcode::I16x8ExtendLowI8x16S => empty_case!(i16x8_extend_low_i8x16_s),
        Opcode::I16x8ExtendHighI8x16S => empty_case!(i16x8_extend_high_i8x16_s),
        Opcode::I16x8ExtendLowI8x16U => empty_case!(i16x8_extend_low_i8x16_u),
        Opcode::I16x8ExtendHighI8x16U => empty_case!(i16x8_extend_high_i8x16_u),
        Opcode::I16x8Shl => empty_case!(i16x8_shl),
        Opcode::I16x8ShrS => empty_case!(i16x8_shr_s),
        Opcode::I16x8ShrU => empty_case!(i16x8_shr_u),
        Opcode::I16x8Add => empty_case!(i16x8_add),
        Opcode::I16x8AddSatS => empty_case!(i16x8_add_sat_s),
        Opcode::I16x8AddSatU => empty_case!(i16x8_add_sat_u),
        Opcode::I16x8Sub => empty_case!(i16x8_sub),
        Opcode::I16x8SubSatS => empty_case!(i16x8_sub_sat_s),
        Opcode::I16x8SubSatU => empty_case!(i16x8_sub_sat_u),
        Opcode::F64x2Nearest => empty_case!(f64x2_nearest),
        Opcode::I16x8Mul => empty_case!(i16x8_mul),
        Opcode::I16x8MinS => empty_case!(i16x8_min_s),
        Opcode::I16x8MinU => empty_case!(i16x8_min_u),
        Opcode::I16x8MaxS => empty_case!(i16x8_max_s),
        Opcode::I16x8MaxU => empty_case!(i16x8_max_u),
        Opcode::I16x8AvgrU => empty_case!(i16x8_avgr_u),
        Opcode::I16x8ExtmulLowI8x16S => empty_case!(i16x8_extmul_low_i8x16_s),
        Opcode::I16x8ExtmulHighI8x16S => empty_case!(i16x8_extmul_high_i8x16_s),
        Opcode::I16x8ExtmulLowI8x16U => empty_case!(i16x8_extmul_low_i8x16_u),
        Opcode::I16x8ExtmulHighI8x16U => empty_case!(i16x8_extmul_high_i8x16_u),
        Opcode::I32x4Abs => empty_case!(i32x4_abs),
        Opcode::I32x4Neg => empty_case!(i32x4_neg),
        Opcode::I32x4AllTrue => empty_case!(i32x4_all_true),
        Opcode::I32x4Bitmask => empty_case!(i32x4_bitmask),
        Opcode::I32x4ExtendLowI16x8S => empty_case!(i32x4_extend_low_i16x8_s),
        Opcode::I32x4ExtendHighI16x8S => empty_case!(i32x4_extend_high_i16x8_s),
        Opcode::I32x4ExtendLowI16x8U => empty_case!(i32x4_extend_low_i16x8_u),
        Opcode::I32x4ExtendHighI16x8U => empty_case!(i32x4_extend_high_i16x8_u),
        Opcode::I32x4Shl => empty_case!(i32x4_shl),
        Opcode::I32x4ShrS => empty_case!(i32x4_shr_s),
        Opcode::I32x4ShrU => empty_case!(i32x4_shr_u),
        Opcode::I32x4Add => empty_case!(i32x4_add),
        Opcode::I32x4Sub => empty_case!(i32x4_sub),
        Opcode::I32x4Mul => empty_case!(i32x4_mul),
        Opcode::I32x4MinS => empty_case!(i32x4_min_s),
        Opcode::I32x4MinU => empty_case!(i32x4_min_u),
        Opcode::I32x4MaxS => empty_case!(i32x4_max_s),
        Opcode::I32x4MaxU => empty_case!(i32x4_max_u),
        Opcode::I32x4DotI16x8S => empty_case!(i32x4_dot_i16x8_s),
        Opcode::I32x4ExtmulLowI16x8S => empty_case!(i32x4_extmul_low_i16x8_s),
        Opcode::I32x4ExtmulHighI16x8S => empty_case!(i32x4_extmul_high_i16x8_s),
        Opcode::I32x4ExtmulLowI16x8U => empty_case!(i32x4_extmul_low_i16x8_u),
        Opcode::I32x4ExtmulHighI16x8U => empty_case!(i32x4_extmul_high_i16x8_u),
        Opcode::I64x2Abs => empty_case!(i64x2_abs),
        Opcode::I64x2Neg => empty_case!(i64x2_neg),
        Opcode::I64x2AllTrue => empty_case!(i64x2_all_true),
        Opcode::I64x2Bitmask => empty_case!(i64x2_bitmask),
        Opcode::I64x2ExtendLowI32x4S => empty_case!(i64x2_extend_low_i32x4_s),
        Opcode::I64x2ExtendHighI32x4S => empty_case!(i64x2_extend_high_i32x4_s),
        Opcode::I64x2ExtendLowI32x4U => empty_case!(i64x2_extend_low_i32x4_u),
        Opcode::I64x2ExtendHighI32x4U => empty_case!(i64x2_extend_high_i32x4_u),
        Opcode::I64x2Shl => empty_case!(i64x2_shl),
        Opcode::I64x2ShrS => empty_case!(i64x2_shr_s),
        Opcode::I64x2ShrU => empty_case!(i64x2_shr_u),
        Opcode::I64x2Add => empty_case!(i64x2_add),
        Opcode::I64x2Sub => empty_case!(i64x2_sub),
        Opcode::I64x2Mul => empty_case!(i64x2_mul),
        Opcode::I64x2Eq => empty_case!(i64x2_eq),
        Opcode::I64x2Ne => empty_case!(i64x2_ne),
        Opcode::I64x2LtS => empty_case!(i64x2_lt_s),
        Opcode::I64x2GtS => empty_case!(i64x2_gt_s),
        Opcode::I64x2LeS => empty_case!(i64x2_le_s),
        Opcode::I64x2GeS => empty_case!(i64x2_ge_s),
        Opcode::I64x2ExtmulLowI32x4S => empty_case!(i64x2_extmul_low_i32x4_s),
        Opcode::I64x2ExtmulHighI32x4S => empty_case!(i64x2_extmul_high_i32x4_s),
        Opcode::I64x2ExtmulLowI32x4U => empty_case!(i64x2_extmul_low_i32x4_u),
        Opcode::I64x2ExtmulHighI32x4U => empty_case!(i64x2_extmul_high_i32x4_u),
        Opcode::F32x4Abs => empty_case!(f32x4_abs),
        Opcode::F32x4Neg => empty_case!(f32x4_neg),
        Opcode::F32x4Sqrt => empty_case!(f32x4_sqrt),
        Opcode::F32x4Add => empty_case!(f32x4_add),
        Opcode::F32x4Sub => empty_case!(f32x4_sub),
        Opcode::F32x4Mul => empty_case!(f32x4_mul),
        Opcode::F32x4Div => empty_case!(f32x4_div),
        Opcode::F32x4Min => empty_case!(f32x4_min),
        Opcode::F32x4Max => empty_case!(f32x4_max),
        Opcode::F32x4Pmin => empty_case!(f32x4_pmin),
        Opcode::F32x4Pmax => empty_case!(f32x4_pmax),
        Opcode::F64x2Abs => empty_case!(f64x2_abs),
        Opcode::F64x2Neg => empty_case!(f64x2_neg),
        Opcode::F64x2Sqrt => empty_case!(f64x2_sqrt),
        Opcode::F64x2Add => empty_case!(f64x2_add),
        Opcode::F64x2Sub => empty_case!(f64x2_sub),
        Opcode::F64x2Mul => empty_case!(f64x2_mul),
        Opcode::F64x2Div => empty_case!(f64x2_div),
        Opcode::F64x2Min => empty_case!(f64x2_min),
        Opcode::F64x2Max => empty_case!(f64x2_max),
        Opcode::F64x2Pmin => empty_case!(f64x2_pmin),
        Opcode::F64x2Pmax => empty_case!(f64x2_pmax),
        Opcode::I32x4TruncSatF32x4S => empty_case!(i32x4_trunc_sat_f32x4_s),
        Opcode::I32x4TruncSatF32x4U => empty_case!(i32x4_trunc_sat_f32x4_u),
        Opcode::F32x4ConvertI32x4S => empty_case!(f32x4_convert_i32x4_s),
        Opcode::F32x4ConvertI32x4U => empty_case!(f32x4_convert_i32x4_u),
        Opcode::I32x4TruncSatF64x2SZero => empty_case!(i32x4_trunc_sat_f64x2_s_zero),
        Opcode::I32x4TruncSatF64x2UZero => empty_case!(i32x4_trunc_sat_f64x2_u_zero),
        Opcode::F64x2ConvertLowI32x4S => empty_case!(f64x2_convert_low_i32x4_s),
        Opcode::F64x2ConvertLowI32x4U => empty_case!(f64x2_convert_low_i32x4_u),
        Opcode::I8x16RelaxedSwizzle => empty_case!(i8x16_relaxed_swizzle),
        Opcode::I32x4RelaxedTruncF32x4S => empty_case!(i32x4_relaxed_trunc_f32x4_s),
        Opcode::I32x4RelaxedTruncF32x4U => empty_case!(i32x4_relaxed_trunc_f32x4_u),
        Opcode::I32x4RelaxedTruncF64x2SZero => {
            empty_case!(i32x4_relaxed_trunc_f64x2_s_zero)
        }
        Opcode::I32x4RelaxedTruncF64x2UZero => {
            empty_case!(i32x4_relaxed_trunc_f64x2_u_zero)
        }
        Opcode::F32x4RelaxedMadd => empty_case!(f32x4_relaxed_madd),
        Opcode::F32x4RelaxedNmadd => empty_case!(f32x4_relaxed_nmadd),
        Opcode::F64x2RelaxedMadd => empty_case!(f64x2_relaxed_madd),
        Opcode::F64x2RelaxedNmadd => empty_case!(f64x2_relaxed_nmadd),
        Opcode::I8x16RelaxedLaneselect => empty_case!(i8x16_relaxed_laneselect),
        Opcode::I16x8RelaxedLaneselect => empty_case!(i16x8_relaxed_laneselect),
        Opcode::I32x4RelaxedLaneselect => empty_case!(i32x4_relaxed_laneselect),
        Opcode::I64x2RelaxedLaneselect => empty_case!(i64x2_relaxed_laneselect),
        Opcode::F32x4RelaxedMin => empty_case!(f32x4_relaxed_min),
        Opcode::F32x4RelaxedMax => empty_case!(f32x4_relaxed_max),
        Opcode::F64x2RelaxedMin => empty_case!(f64x2_relaxed_min),
        Opcode::F64x2RelaxedMax => empty_case!(f64x2_relaxed_max),
        Opcode::I16x8RelaxedQ15mulrS => empty_case!(i16x8_relaxed_q15mulr_s),
        Opcode::I16x8RelaxedDotI8x16I7x16S => empty_case!(i16x8_relaxed_dot_i8x16_i7x16_s),
        Opcode::I32x4RelaxedDotI8x16I7x16AddS => {
            empty_case!(i32x4_relaxed_dot_i8x16_i7x16_add_s)
        }
        Opcode::MemoryAtomicNotify => mem_op!(memory_atomic_notify),
        Opcode::MemoryAtomicWait32 => mem_op!(memory_atomic_wait32),
        Opcode::MemoryAtomicWait64 => mem_op!(memory_atomic_wait64),
        Opcode::I32AtomicLoad => mem_op!(i32_atomic_load),
        Opcode::I64AtomicLoad => mem_op!(i64_atomic_load),
        Opcode::I32AtomicLoad8U => mem_op!(i32_atomic_load8_u),
        Opcode::I32AtomicLoad16U => mem_op!(i32_atomic_load16_u),
        Opcode::I64AtomicLoad8U => mem_op!(i64_atomic_load8_u),
        Opcode::I64AtomicLoad16U => mem_op!(i64_atomic_load16_u),
        Opcode::I64AtomicLoad32U => mem_op!(i64_atomic_load32_u),
        Opcode::I32AtomicStore => mem_op!(i32_atomic_store),
        Opcode::I64AtomicStore => mem_op!(i64_atomic_store),
        Opcode::I32AtomicStore8U => mem_op!(i32_atomic_store8_u),
        Opcode::I32AtomicStore16U => mem_op!(i32_atomic_store16_u),
        Opcode::I64AtomicStore8U => mem_op!(i64_atomic_store8_u),
        Opcode::I64AtomicStore16U => mem_op!(i64_atomic_store16_u),
        Opcode::I64AtomicStore32U => mem_op!(i64_atomic_store32_u),
        Opcode::I32AtomicRmwAdd => mem_op!(i32_atomic_rmw_add),
        Opcode::I64AtomicRmwAdd => mem_op!(i64_atomic_rmw_add),
        Opcode::I32AtomicRmw8AddU => mem_op!(i32_atomic_rmw8_add_u),
        Opcode::I32AtomicRmw16AddU => mem_op!(i32_atomic_rmw16_add_u),
        Opcode::I64AtomicRmw8AddU => mem_op!(i64_atomic_rmw8_add_u),
        Opcode::I64AtomicRmw16AddU => mem_op!(i64_atomic_rmw16_add_u),
        Opcode::I64AtomicRmw32AddU => mem_op!(i64_atomic_rmw32_add_u),
        Opcode::I32AtomicRmwSub => mem_op!(i32_atomic_rmw_sub),
        Opcode::I64AtomicRmwSub => mem_op!(i64_atomic_rmw_sub),
        Opcode::I32AtomicRmw8SubU => mem_op!(i32_atomic_rmw8_sub_u),
        Opcode::I32AtomicRmw16SubU => mem_op!(i32_atomic_rmw16_sub_u),
        Opcode::I64AtomicRmw8SubU => mem_op!(i64_atomic_rmw8_sub_u),
        Opcode::I64AtomicRmw16SubU => mem_op!(i64_atomic_rmw16_sub_u),
        Opcode::I64AtomicRmw32SubU => mem_op!(i64_atomic_rmw32_sub_u),
        Opcode::I32AtomicRmwAnd => mem_op!(i32_atomic_rmw_and),
        Opcode::I64AtomicRmwAnd => mem_op!(i64_atomic_rmw_and),
        Opcode::I32AtomicRmw8AndU => mem_op!(i32_atomic_rmw8_and_u),
        Opcode::I32AtomicRmw16AndU => mem_op!(i32_atomic_rmw16_and_u),
        Opcode::I64AtomicRmw8AndU => mem_op!(i64_atomic_rmw8_and_u),
        Opcode::I64AtomicRmw16AndU => mem_op!(i64_atomic_rmw16_and_u),
        Opcode::I64AtomicRmw32AndU => mem_op!(i64_atomic_rmw32_and_u),
        Opcode::I32AtomicRmwOr => mem_op!(i32_atomic_rmw_or),
        Opcode::I64AtomicRmwOr => mem_op!(i64_atomic_rmw_or),
        Opcode::I32AtomicRmw8OrU => mem_op!(i32_atomic_rmw8_or_u),
        Opcode::I32AtomicRmw16OrU => mem_op!(i32_atomic_rmw16_or_u),
        Opcode::I64AtomicRmw8OrU => mem_op!(i64_atomic_rmw8_or_u),
        Opcode::I64AtomicRmw16OrU => mem_op!(i64_atomic_rmw16_or_u),
        Opcode::I64AtomicRmw32OrU => mem_op!(i64_atomic_rmw32_or_u),
        Opcode::I32AtomicRmwXor => mem_op!(i32_atomic_rmw_xor),
        Opcode::I64AtomicRmwXor => mem_op!(i64_atomic_rmw_xor),
        Opcode::I32AtomicRmw8XorU => mem_op!(i32_atomic_rmw8_xor_u),
        Opcode::I32AtomicRmw16XorU => mem_op!(i32_atomic_rmw16_xor_u),
        Opcode::I64AtomicRmw8XorU => mem_op!(i64_atomic_rmw8_xor_u),
        Opcode::I64AtomicRmw16XorU => mem_op!(i64_atomic_rmw16_xor_u),
        Opcode::I64AtomicRmw32XorU => mem_op!(i64_atomic_rmw32_xor_u),
        Opcode::I32AtomicRmwXchg => mem_op!(i32_atomic_rmw_xchg),
        Opcode::I64AtomicRmwXchg => mem_op!(i64_atomic_rmw_xchg),
        Opcode::I32AtomicRmw8XchgU => mem_op!(i32_atomic_rmw8_xchg_u),
        Opcode::I32AtomicRmw16XchgU => mem_op!(i32_atomic_rmw16_xchg_u),
        Opcode::I64AtomicRmw8XchgU => mem_op!(i64_atomic_rmw8_xchg_u),
        Opcode::I64AtomicRmw16XchgU => mem_op!(i64_atomic_rmw16_xchg_u),
        Opcode::I64AtomicRmw32XchgU => mem_op!(i64_atomic_rmw32_xchg_u),
        Opcode::I32AtomicRmwCmpxchg => mem_op!(i32_atomic_rmw_cmpxchg),
        Opcode::I64AtomicRmwCmpxchg => mem_op!(i64_atomic_rmw_cmpxchg),
        Opcode::I32AtomicRmw8CmpxchgU => mem_op!(i32_atomic_rmw8_cmpxchg_u),
        Opcode::I32AtomicRmw16CmpxchgU => mem_op!(i32_atomic_rmw16_cmpxchg_u),
        Opcode::I64AtomicRmw8CmpxchgU => mem_op!(i64_atomic_rmw8_cmpxchg_u),
        Opcode::I64AtomicRmw16CmpxchgU => mem_op!(i64_atomic_rmw16_cmpxchg_u),
        Opcode::I64AtomicRmw32CmpxchgU => mem_op!(i64_atomic_rmw32_cmpxchg_u),
    };

    Ok((input, parser))
}

/// A [`nom::Parser`] implementation for parsing a [WebAssembly instruction].
///
/// See the documentation for the [`ParseInstr::into_parser()`] method for more information.
///
/// [WebAssembly instruction]: https://webassembly.github.io/spec/core/binary/instructions.html
pub struct InstrParser<'a, E, P>
where
    E: ErrorSource<'a>,
    P: ParseInstr<'a, E>,
{
    parser: P,
    _marker: core::marker::PhantomData<dyn nom::Parser<&'a [u8], (), E>>,
}

impl<'a, E, P> InstrParser<'a, E, P>
where
    E: ErrorSource<'a>,
    P: ParseInstr<'a, E>,
{
    pub(in crate::isa) fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: core::marker::PhantomData,
        }
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn into_parse_instr(self) -> P {
        self.parser
    }
}

impl<'a, E, P> nom::Parser<&'a [u8], (), E> for InstrParser<'a, E, P>
where
    E: ErrorSource<'a>,
    P: ParseInstr<'a, E>,
{
    #[inline]
    fn parse(&mut self, input: &'a [u8]) -> crate::Parsed<'a, (), E> {
        parse(input, &mut self.parser).map(|(input, _)| (input, ()))
    }
}

impl<'a, E, P> core::fmt::Debug for InstrParser<'a, E, P>
where
    E: ErrorSource<'a>,
    P: ParseInstr<'a, E> + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InstrParser")
            .field("parser", &self.parser)
            .finish()
    }
}
