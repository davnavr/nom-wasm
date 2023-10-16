/// Applies a macro to each of the instructions recognized by [`nom-wasm`](crate), where the
/// `$called_macro` is in the form:
///
/// ```no_run
/// macro_rules! called_macro {
///     ($(
///         $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($field_name:ident: $field_type:ident),+ })? $snake_ident:ident;
///     )*) => {
///         // Add your macro content here
///     };
/// }
/// ```
///
/// Where:
/// - `$opcode_case` is the name of the `InstrKind` case corresponding to the instruction's opcode.
/// - `$wasm_name` is a string literal corresponding to the name of instruction in the WebAssembly
///   Text Format.
macro_rules! all {
    ($called_macro:ident) => {
        $called_macro! {
            // MVP, Control

            /*
            ["does nothing." @ "core/syntax/instructions.html#syntax-instr-control"]
            */
            Byte /*mvp*/ "unreachable" Unreachable unreachable;
            Byte /*mvp*/ "nop" Nop nop;
            Byte /*mvp*/ "block" Block { block_type: BlockType } block;
            Byte /*mvp*/ "loop" Loop { block_type: BlockType }  r#loop;
            Byte /*mvp*/ "if" If { block_type: BlockType }  r#if;
            Byte /*mvp*/ "else" Else r#else;
            Byte /*mvp*/ "end" End end;
            Byte /*mvp*/ "br" Br { target: LabelIdx } br;
            Byte /*mvp*/ "br_if" BrIf { target: LabelIdx } br_if;
            Byte /*mvp*/ "br_table" BrTable br_table; // TODO: BrTableTargets
            Byte /*mvp*/ "return" Return r#return;
            Byte /*mvp*/ "call" Call { callee: FuncIdx } call;
            Byte /*mvp*/ "call_indirect" CallIndirect { signature: TypeIdx, table: TableIdx } call_indirect;

            // MVP, Parametric

            Byte /*mvp*/ "drop" Drop r#drop;
            Byte /*mvp*/ "select" Select select;
            Byte /*mvp*/ "select" SelectTyped select_typed; // TODO: SelectTypes

            // MVP, Variable

            Byte /*mvp*/ "local.get" LocalGet { local: LocalIdx } local_get;
            Byte /*mvp*/ "local.set" LocalSet { local: LocalIdx } local_set;
            Byte /*mvp*/ "local.tee" LocalTee { local: LocalIdx } local_tee;
            Byte /*mvp*/ "global.get" GlobalGet { r#global: GlobalIdx } global_get;
            Byte /*mvp*/ "global.set" GlobalSet { r#global: GlobalIdx } global_set;

            // MVP, Memory

            Byte /*mvp*/ "i32.load" I32Load { arg: MemArg } i32_load;
            Byte /*mvp*/ "i64.load" I64Load { arg: MemArg } i64_load;
            Byte /*mvp*/ "f32.load" F32Load { arg: MemArg } f32_load;
            Byte /*mvp*/ "f64.load" F64Load { arg: MemArg } f64_load;
            Byte /*mvp*/ "i32.load8_s" I32Load8S { arg: MemArg } i32_load8_s;
            Byte /*mvp*/ "i32.load8_u" I32Load8U { arg: MemArg } i32_load8_u;
            Byte /*mvp*/ "i32.load16_s" I32Load16S { arg: MemArg } i32_load16_s;
            Byte /*mvp*/ "i32.load16_u" I32Load16U { arg: MemArg } i32_load16_u;
            Byte /*mvp*/ "i64.load8_s" I64Load8S { arg: MemArg } i64_load8_s;
            Byte /*mvp*/ "i64.load8_u" I64Load8U { arg: MemArg } i64_load8_u;
            Byte /*mvp*/ "i64.load16_s" I64Load16S { arg: MemArg } i64_load16_s;
            Byte /*mvp*/ "i64.load16_u" I64Load16U { arg: MemArg } i64_load16_u;
            Byte /*mvp*/ "i64.load32_s" I64Load32S { arg: MemArg } i64_load32_s;
            Byte /*mvp*/ "i64.load32_u" I64Load32U { arg: MemArg } i64_load32_u;
            Byte /*mvp*/ "i32.store" I32Store { arg: MemArg } i32_store;
            Byte /*mvp*/ "i64.store" I64Store { arg: MemArg } i64_store;
            Byte /*mvp*/ "f32.store" F32Store { arg: MemArg } f32_store;
            Byte /*mvp*/ "f64.store" F64Store { arg: MemArg } f64_store;
            Byte /*mvp*/ "i32.store8" I32Store8 { arg: MemArg } i32_store8;
            Byte /*mvp*/ "i32.store16" I32Store16 { arg: MemArg } i32_store16;
            Byte /*mvp*/ "i64.store8" I64Store8 { arg: MemArg } i64_store8;
            Byte /*mvp*/ "i64.store16" I64Store16 { arg: MemArg } i64_store16;
            Byte /*mvp*/ "i64.store32" I64Store32 { arg: MemArg } i64_store32;
            Byte /*mvp*/ "memory.size" MemorySize { memory: MemIdx } memory_size;
            Byte /*mvp*/ "memory.grow" MemoryGrow { memory: MemIdx } memory_grow;

            // MVP, Numeric

            Byte /*mvp*/ "i32.const" I32Const { n: i32 } i32_const;
            Byte /*mvp*/ "i64.const" I64Const { n: i64 } i64_const;
            Byte /*mvp*/ "f32.const" F32Const { z: F32 } f32_const;
            Byte /*mvp*/ "f64.const" F64Const { z: F64 } f64_const;
            Byte /*mvp*/ "i32.eqz" I32Eqz i32_eqz;
            Byte /*mvp*/ "i32.eq" I32Eq i32_eq;
            Byte /*mvp*/ "i32.ne" I32Ne i32_ne;
            Byte /*mvp*/ "i32.lt_s" I32LtS i32_lt_s;
            Byte /*mvp*/ "i32.lt_u" I32LtU i32_lt_u;
            Byte /*mvp*/ "i32.gt_s" I32GtS i32_gt_s;
            Byte /*mvp*/ "i32.gt_u" I32GtU i32_gt_u;
            Byte /*mvp*/ "i32.le_s" I32LeS i32_le_s;
            Byte /*mvp*/ "i32.le_u" I32LeU i32_le_u;
            Byte /*mvp*/ "i32.ge_s" I32GeS i32_lg_s;
            Byte /*mvp*/ "i32.ge_u" I32GeU i32_ge_u;
            Byte /*mvp*/ "i64.eqz" I64Eqz i64_eqz;
            Byte /*mvp*/ "i64.eq" I64Eq i64_eq;
            Byte /*mvp*/ "i64.ne" I64Ne i64_ne;
            Byte /*mvp*/ "i64.lt_s" I64LtS i64_lt_s;
            Byte /*mvp*/ "i64.lt_u" I64LtU i64_lt_u;
            Byte /*mvp*/ "i64.gt_s" I64GtS i64_gt_s;
            Byte /*mvp*/ "i64.gt_u" I64GtU i64_gt_u;
            Byte /*mvp*/ "i64.le_s" I64LeS i64_le_s;
            Byte /*mvp*/ "i64.le_u" I64LeU i64_le_u;
            Byte /*mvp*/ "i64.ge_s" I64GeS i64_ge_s;
            Byte /*mvp*/ "i64.ge_u" I64GeU i64_ge_u;
            Byte /*mvp*/ "f32.eq" F32Eq f32_eq;
            Byte /*mvp*/ "f32.ne" F32Ne f32_ne;
            Byte /*mvp*/ "f32.lt" F32Lt f32_lt;
            Byte /*mvp*/ "f32.gt" F32Gt f32_gt;
            Byte /*mvp*/ "f32.le" F32Le f32_le;
            Byte /*mvp*/ "f32.ge" F32Ge f32_ge;
            Byte /*mvp*/ "f64.eq" F64Eq f64_eq;
            Byte /*mvp*/ "f64.ne" F64Ne f64_ne;
            Byte /*mvp*/ "f64.lt" F64Lt f64_lt;
            Byte /*mvp*/ "f64.gt" F64Gt f64_gt;
            Byte /*mvp*/ "f64.le" F64Le f64_le;
            Byte /*mvp*/ "f64.ge" F64Ge f64_ge;
            Byte /*mvp*/ "i32.clz" I32Clz i32_clz;
            Byte /*mvp*/ "i32.ctz" I32Ctz i32_ctz;
            Byte /*mvp*/ "i32.popcnt" I32Popcnt i32_popcnt;
            Byte /*mvp*/ "i32.add" I32Add i32_add;
            Byte /*mvp*/ "i32.sub" I32Sub i32_sub;
            Byte /*mvp*/ "i32.mul" I32Mul i32_mul;
            Byte /*mvp*/ "i32.div_s" I32DivS i32_div_s;
            Byte /*mvp*/ "i32.div_u" I32DivU i32_div_u;
            Byte /*mvp*/ "i32.rem_s" I32RemS i32_rem_s;
            Byte /*mvp*/ "i32.rem_u" I32RemU i32_rem_u;
            Byte /*mvp*/ "i32.and" I32And i32_and;
            Byte /*mvp*/ "i32.or" I32Or i32_or;
            Byte /*mvp*/ "i32.xor" I32Xor i32_xor;
            Byte /*mvp*/ "i32.shl" I32Shl i32_shl;
            Byte /*mvp*/ "i32.shr_s" I32ShrS i32_shr_s;
            Byte /*mvp*/ "i32.shr_u" I32ShrU i32_shr_u;
            Byte /*mvp*/ "i32.rotl" I32Rotl i32_rotl;
            Byte /*mvp*/ "i32.rotr" I32Rotr i32_rotr;
            Byte /*mvp*/ "i64.clz" I64Clz i64_clz;
            Byte /*mvp*/ "i64.ctz" I64Ctz i64_ctz;
            Byte /*mvp*/ "i64.popcnt" I64Popcnt i64_popcnt;
            Byte /*mvp*/ "i64.add" I64Add i64_add;
            Byte /*mvp*/ "i64.sub" I64Sub i64_sub;
            Byte /*mvp*/ "i64.mul" I64Mul i64_mul;
            Byte /*mvp*/ "i64.div_s" I64DivS i64_div_s;
            Byte /*mvp*/ "i64.div_u" I64DivU i64_div_u;
            Byte /*mvp*/ "i64.rem_s" I64RemS i64_rem_s;
            Byte /*mvp*/ "i64.rem_u" I64RemU i64_rem_u;
            Byte /*mvp*/ "i64.and" I64And i64_and;
            Byte /*mvp*/ "i64.or" I64Or i64_or;
            Byte /*mvp*/ "i64.xor" I64Xor i64_xor;
            Byte /*mvp*/ "i64.shl" I64Shl i64_shl;
            Byte /*mvp*/ "i64.shr_s" I64ShrS i64_shr_s;
            Byte /*mvp*/ "i64.shr_u" I64ShrU i64_shr_u;
            Byte /*mvp*/ "i64.rotl" I64Rotl i64_rotl;
            Byte /*mvp*/ "i64.rotr" I64Rotr i64_rotr;
            Byte /*mvp*/ "f32.abs" F32Abs f32_abs;
            Byte /*mvp*/ "f32.neg" F32Neg f32_neg;
            Byte /*mvp*/ "f32.ceil" F32Ceil f32_ceil;
            Byte /*mvp*/ "f32.floor" F32Floor f32_floor;
            Byte /*mvp*/ "f32.trunc" F32Trunc f32_trunc;
            Byte /*mvp*/ "f32.nearest" F32Nearest f32_nearest;
            Byte /*mvp*/ "f32.sqrt" F32Sqrt f32_sqrt;
            Byte /*mvp*/ "f32.add" F32Add f32_add;
            Byte /*mvp*/ "f32.sub" F32Sub f32_sub;
            Byte /*mvp*/ "f32.mul" F32Mul f32_mul;
            Byte /*mvp*/ "f32.div" F32Div f32_div;
            Byte /*mvp*/ "f32.min" F32Min f32_min;
            Byte /*mvp*/ "f32.max" F32Max f32_max;
            Byte /*mvp*/ "f32.copysign" F32Copysign f32_copysign;
            Byte /*mvp*/ "f64.abs" F64Abs f64_abs;
            Byte /*mvp*/ "f64.neg" F64Neg f64_neg;
            Byte /*mvp*/ "f64.ceil" F64Ceil f64_ceil;
            Byte /*mvp*/ "f64.floor" F64Floor f64_floor;
            Byte /*mvp*/ "f64.trunc" F64Trunc f64_trunc;
            Byte /*mvp*/ "f64.nearest" F64Nearest f64_nearest;
            Byte /*mvp*/ "f64.sqrt" F64Sqrt f64_sqrt;
            Byte /*mvp*/ "f64.add" F64Add f64_add;
            Byte /*mvp*/ "f64.sub" F64Sub f64_sub;
            Byte /*mvp*/ "f64.mul" F64Mul f64_mul;
            Byte /*mvp*/ "f64.div" F64Div f64_div;
            Byte /*mvp*/ "f64.min" F64Min f64_min;
            Byte /*mvp*/ "f64.max" F64Max f64_max;
            Byte /*mvp*/ "f64.copysign" F64Copysign f64_copysign;
            Byte /*mvp*/ "i32.wrap_i64" I32WrapI64 i32_wrap_i64;
            Byte /*mvp*/ "i32.trunc_f32_s" I32TruncF32S i32_trunc_f32_s;
            Byte /*mvp*/ "i32.trunc_f32_u" I32TruncF32U i32_trunc_f32_u;
            Byte /*mvp*/ "i32.trunc_f64_s" I32TruncF64S i32_trunc_f64_s;
            Byte /*mvp*/ "i32.trunc_f64_u" I32TruncF64U i32_trunc_f64_u;
            Byte /*mvp*/ "i64.extend_i32_s" I64ExtendI32S i64_extend_i32_s;
            Byte /*mvp*/ "i64.extend_i32_u" I64ExtendI32U i64_extend_i32_u;
            Byte /*mvp*/ "i64.trunc_f32_s" I64TruncF32S i64_trunc_f32_s;
            Byte /*mvp*/ "i64.trunc_f32_u" I64TruncF32U i64_trunc_f32_u;
            Byte /*mvp*/ "i64.trunc_f64_s" I64TruncF64S i64_trunc_f64_s;
            Byte /*mvp*/ "i64.trunc_f64_u" I64TruncF64U i64_trunc_f64_u;
            Byte /*mvp*/ "f32.convert_i32_s" F32ConvertI32S f32_convert_i32_s;
            Byte /*mvp*/ "f32.convert_i32_u" F32ConvertI32U f32_convert_i32_u;
            Byte /*mvp*/ "f32.convert_i64_s" F32ConvertI64S f32_convert_i64_s;
            Byte /*mvp*/ "f32.convert_i64_u" F32ConvertI64U f32_convert_i64_u;
            Byte /*mvp*/ "f32.demote_f64" F32DemoteF64 f32_demote_f64;
            Byte /*mvp*/ "f64.convert_i32_s" F64ConvertI32S f64_convert_i32_s;
            Byte /*mvp*/ "f64.convert_i32_u" F64ConvertI32U f64_convert_i32_u;
            Byte /*mvp*/ "f64.convert_i64_s" F64ConvertI64S f64_convert_i64_s;
            Byte /*mvp*/ "f64.convert_i64_u" F64ConvertI64U f64_convert_i64_u;
            Byte /*mvp*/ "f64.promote_f32" F64PromoteF32 f64_promote_f32;
            Byte /*mvp*/ "i32.reinterpret_f32" I32ReinterpretF32 i32_reinterpret_f32;
            Byte /*mvp*/ "i64.reinterpret_f64" I64ReinterpretF64 i64_reinterpret_f64;
            Byte /*mvp*/ "f32.reinterpret_i32" F32ReinterpretI32 f32_reinterpret_i32;
            Byte /*mvp*/ "f64.reinterpret_i64" F64ReinterpretI64 f64_reinterpret_i64;

            // Non-Trapping Float-To-Int, Numeric

            FCPrefixed /*nontrapping_fptoint*/ "i32.trunc_sat_f32_s" I32TruncSatF32S i32_trunc_sat_f32_s;
            FCPrefixed /*nontrapping_fptoint*/ "i32.trunc_sat_f32_u" I32TruncSatF32U i32_trunc_sat_f32_u;
            FCPrefixed /*nontrapping_fptoint*/ "i32.trunc_sat_f64_s" I32TruncSatF64S i32_trunc_sat_f64_s;
            FCPrefixed /*nontrapping_fptoint*/ "i32.trunc_sat_f64_u" I32TruncSatF64U i32_trunc_sat_f64_u;
            FCPrefixed /*nontrapping_fptoint*/ "i64.trunc_sat_f32_s" I64TruncSatF32S i64_trunc_sat_f32_s;
            FCPrefixed /*nontrapping_fptoint*/ "i64.trunc_sat_f32_u" I64TruncSatF32U i64_trunc_sat_f32_u;
            FCPrefixed /*nontrapping_fptoint*/ "i64.trunc_sat_f64_s" I64TruncSatF64S i64_trunc_sat_f64_s;
            FCPrefixed /*nontrapping_fptoint*/ "i64.trunc_sat_f64_u" I64TruncSatF64U i64_trunc_sat_f64_u;

            // Sign-Extension Operators, Numeric

            Byte /*sign_ext*/ "i32.extend8_s" I32Extend8S i32_extend8_s;
            Byte /*sign_ext*/ "i32.extend16_s" I32Extend16S i32_extend16_s;
            Byte /*sign_ext*/ "i64.extend8_s" I64Extend8S i64_extend8_s;
            Byte /*sign_ext*/ "i64.extend16_s" I64Extend16S i64_extend16_s;
            Byte /*sign_ext*/ "i64.extend32_s" I64Extend32S i64_extend32_s;

            // Bulk Memory, Memory

            FCPrefixed /*bulk_memory*/ "memory.copy" MemoryCopy { destination: MemIdx, source: MemIdx } memory_copy;
            FCPrefixed /*bulk_memory*/ "memory.fill" MemoryFill { memory: MemIdx } memory_fill;
            FCPrefixed /*bulk_memory*/ "memory.init" MemoryInit { segment: DataIdx, memory: MemIdx } memory_init;
            FCPrefixed /*bulk_memory*/ "data.drop" DataDrop { segment: DataIdx } data_drop;

            // Bulk Memory, Table

            FCPrefixed /*bulk_memory*/ "table.copy" TableCopy { destination: TableIdx, source: TableIdx } table_copy;
            FCPrefixed /*bulk_memory*/ "table.init" TableInit { segment: ElemIdx, table: TableIdx } table_init;
            FCPrefixed /*bulk_memory*/ "elem.drop" ElemDrop { segment: ElemIdx } elem_drop;

            // Reference Type, Reference

            Byte /*reference_types*/ "ref.null" RefNull { reference_type: RefType } ref_null;
            Byte /*reference_types*/ "ref.is_null" RefIsNull ref_is_null;
            Byte /*reference_types*/ "ref.func" RefFunc { target: FuncIdx } ref_func;

            // Reference Type, Table

            Byte /*reference_types*/ "table.get" TableGet { table: TableIdx } table_get;
            Byte /*reference_types*/ "table.set" TableSet { table: TableIdx } table_set;
            FCPrefixed /*reference_types*/ "table.size" TableSize { table: TableIdx } table_size;
            FCPrefixed /*reference_types*/ "table.grow" TableGrow { table: TableIdx } table_grow;
            FCPrefixed /*reference_types*/ "table.fill" TableFill { table: TableIdx } table_fill;

            // Fixed Width SIMD, Memory

            V128 /*simd128*/ "v128.load" V128Load { arg: MemArg } v128_load;
            V128 /*simd128*/ "v128.load8x8_s" V128Load8x8S { arg: MemArg } v128_load8x8_s;
            V128 /*simd128*/ "v128.load8x8_u" V128Load8x8U { arg: MemArg } v128_load8x8_u;
            V128 /*simd128*/ "v128.load16x4_s" V128Load16x4S { arg: MemArg } v128_load16x4_s;
            V128 /*simd128*/ "v128.load16x4_u" V128Load16x4U { arg: MemArg } v128_load16x4_u;
            V128 /*simd128*/ "v128.load32x2_s" V128Load32x2S { arg: MemArg } v128_load32x2_s;
            V128 /*simd128*/ "v128.load32x2_u" V128Load32x2U { arg: MemArg } v128_load32x2_u;
            V128 /*simd128*/ "v128.load8_splat" V128Load8Splat { arg: MemArg } v128_load8_splat;
            V128 /*simd128*/ "v128.load16_splat" V128Load16Splat { arg: MemArg } v128_load16_splat;
            V128 /*simd128*/ "v128.load32_splat" V128Load32Splat { arg: MemArg } v128_load32_splat;
            V128 /*simd128*/ "v128.load64_splat" V128Load64Splat { arg: MemArg } v128_load64_splat;
            V128 /*simd128*/ "v128.load32_zero" V128Load32Zero { arg: MemArg } v128_load32_zero;
            V128 /*simd128*/ "v128.load64_zero" V128Load64Zero { arg: MemArg } v128_load64_zero;
            V128 /*simd128*/ "v128.store" V128Store { arg: MemArg } v128_store;
            V128 /*simd128*/ "v128.load8_lane" V128Load8Lane { arg: MemArg, lane: LaneIdx } v128_load8_lane;
            V128 /*simd128*/ "v128.load16_lane" V128Load16Lane { arg: MemArg, lane: LaneIdx } v128_load16_lane;
            V128 /*simd128*/ "v128.load32_lane" V128Load32Lane { arg: MemArg, lane: LaneIdx } v128_load32_lane;
            V128 /*simd128*/ "v128.load64_lane" V128Load64Lane { arg: MemArg, lane: LaneIdx } v128_load64_lane;
            V128 /*simd128*/ "v128.store8_lane" V128Store8Lane { arg: MemArg, lane: LaneIdx } v128_store8_lane;
            V128 /*simd128*/ "v128.store16_lane" V128Store16Lane { arg: MemArg, lane: LaneIdx } v128_store16_lane;
            V128 /*simd128*/ "v128.store32_lane" V128Store32Lane { arg: MemArg, lane: LaneIdx } v128_store32_lane;
            V128 /*simd128*/ "v128.store64_lane" V128Store64Lane { arg: MemArg, lane: LaneIdx } v128_store64_lane;

            // Fixed Width SIMD, Vector

            V128 /*simd128*/ "v128.const" V128Const { v: V128 } v128_const;
            V128 /*simd128*/ "i8x16.shuffle" I8x16Shuffle { lanes: V128ShuffleLanes } i8x16_shuffle;
            V128 /*simd128*/ "i8x16.swizzle" I8x16Swizzle i8x16_swizzle;
            V128 /*simd128*/ "i8x16.splat" I8x16Splat i8x16_splat;
            V128 /*simd128*/ "i16x8.splat" I16x8Splat i16x8_splat;
            V128 /*simd128*/ "i32x4.splat" I32x4Splat i32x4_splat;
            V128 /*simd128*/ "i64x2.splat" I64x2Splat i64x2_splat;
            V128 /*simd128*/ "f32x4.splat" F32x4Splat f32x4_splat;
            V128 /*simd128*/ "f64x2.splat" F64x2Splat f64x2_splat;
            V128 /*simd128*/ "i8x16.extract_lane_s" I8x16ExtractLaneS { lane: LaneIdx } i8x16_extract_lane_s;
            V128 /*simd128*/ "i8x16.extract_lane_u" I8x16ExtractLaneU { lane: LaneIdx } i8x16_extract_lane_u;
            V128 /*simd128*/ "i8x16.replace_lane" I8x16ReplaceLane { lane: LaneIdx } i8x16_replace_lane;
            V128 /*simd128*/ "i16x8.extract_lane_s" I16x8ExtractLaneS { lane: LaneIdx } i16x8_extract_lane_s;
            V128 /*simd128*/ "i16x8.extract_lane_u" I16x8ExtractLaneU { lane: LaneIdx } i16x8_extract_lane_u;
            V128 /*simd128*/ "i16x8.replace_lane" I16x8ReplaceLane { lane: LaneIdx } i16x8_replace_lane;
            V128 /*simd128*/ "i32x4.extract_lane" I32x4ExtractLane { lane: LaneIdx } i32x4_extract_lane;
            V128 /*simd128*/ "i32x4.replace_lane" I32x4ReplaceLane { lane: LaneIdx } i32x4_replace_lane;
            V128 /*simd128*/ "i64x2.extract_lane" I64x2ExtractLane { lane: LaneIdx } i64x2_extract_lane;
            V128 /*simd128*/ "i64x2.replace_lane" I64x2ReplaceLane { lane: LaneIdx } i64x2_replace_lane;
            V128 /*simd128*/ "f32x4.extract_lane" F32x4ExtractLane { lane: LaneIdx } f32x4_extract_lane;
            V128 /*simd128*/ "f32x4.replace_lane" F32x4ReplaceLane { lane: LaneIdx } f32x4_replace_lane;
            V128 /*simd128*/ "f64x4.extract_lane" F64x2ExtractLane { lane: LaneIdx } f64x4_extract_lane;
            V128 /*simd128*/ "f64x4.replace_lane" F64x2ReplaceLane { lane: LaneIdx } f64x4_replace_lane;
            V128 /*simd128*/ "i8x16.eq" I8x16Eq i8x16_eq;
            V128 /*simd128*/ "i8x16.ne" I8x16Ne i8x16_ne;
            V128 /*simd128*/ "i8x16.lt_s" I8x16LtS i8x16_lt_s;
            V128 /*simd128*/ "i8x16.lt_u" I8x16LtU i8x16_lt_u;
            V128 /*simd128*/ "i8x16.gt_s" I8x16GtS i8x16_gt_s;
            V128 /*simd128*/ "i8x16.gt_u" I8x16GtU i8x16_gt_u;
            V128 /*simd128*/ "i8x16.le_s" I8x16LeS i8x16_le_s;
            V128 /*simd128*/ "i8x16.le_u" I8x16LeU i8x16_le_u;
            V128 /*simd128*/ "i8x16.ge_s" I8x16GeS i8x16_ge_s;
            V128 /*simd128*/ "i8x16.ge_u" I8x16GeU i8x16_ge_u;
            V128 /*simd128*/ "i16x8.eq" I16x8Eq i16x8_eq;
            V128 /*simd128*/ "i16x8.ne" I16x8Ne i16x8_ne;
            V128 /*simd128*/ "i16x8.lt_s" I16x8LtS i16x8_lt_s;
            V128 /*simd128*/ "i16x8.lt_u" I16x8LtU i16x8_lt_u;
            V128 /*simd128*/ "i16x8.gt_s" I16x8GtS i16x8_gt_s;
            V128 /*simd128*/ "i16x8.gt_u" I16x8GtU i16x8_gt_u;
            V128 /*simd128*/ "i16x8.le_s" I16x8LeS i16x8_le_s;
            V128 /*simd128*/ "i16x8.le_u" I16x8LeU i16x8_le_u;
            V128 /*simd128*/ "i16x8.ge_s" I16x8GeS i16x8_ge_s;
            V128 /*simd128*/ "i16x8.ge_u" I16x8GeU i16x8_ge_u;
            V128 /*simd128*/ "i32x4.eq" I32x4Eq i32x4_eq;
            V128 /*simd128*/ "i32x4.ne" I32x4Ne i32x4_ne;
            V128 /*simd128*/ "i32x4.lt_s" I32x4LtS i32x4_lt_s;
            V128 /*simd128*/ "i32x4.lt_u" I32x4LtU i32x4_lt_u;
            V128 /*simd128*/ "i32x4.gt_s" I32x4GtS i32x4_gt_s;
            V128 /*simd128*/ "i32x4.gt_u" I32x4GtU i32x4_gt_u;
            V128 /*simd128*/ "i32x4.le_s" I32x4LeS i32x4_le_s;
            V128 /*simd128*/ "i32x4.le_u" I32x4LeU i32x4_le_u;
            V128 /*simd128*/ "i32x4.ge_s" I32x4GeS i32x4_ge_s;
            V128 /*simd128*/ "i32x4.ge_u" I32x4GeU i32x4_ge_u;
            V128 /*simd128*/ "f32x4.eq" F32x4Eq f32x4_eq;
            V128 /*simd128*/ "f32x4.ne" F32x4Ne f32x4_ne;
            V128 /*simd128*/ "f32x4.lt" F32x4Lt f32x4_lt;
            V128 /*simd128*/ "f32x4.gt" F32x4Gt f32x4_gt;
            V128 /*simd128*/ "f32x4.le" F32x4Le f32x4_le;
            V128 /*simd128*/ "f32x4.ge" F32x4Ge f32x4_ge;
            V128 /*simd128*/ "f64x2.eq" F64x2Eq f64x2_eq;
            V128 /*simd128*/ "f64x2.ne" F64x2Ne f64x2_ne;
            V128 /*simd128*/ "f64x2.lt" F64x2Lt f64x2_lt;
            V128 /*simd128*/ "f64x2.gt" F64x2Gt f64x2_gt;
            V128 /*simd128*/ "f64x2.le" F64x2Le f64x2_le;
            V128 /*simd128*/ "f64x2.ge" F64x2Ge f64x2_ge;
            V128 /*simd128*/ "v128.not" V128Not v128_not;
            V128 /*simd128*/ "v128.and" V128And v128_and;
            V128 /*simd128*/ "v128.andnot" V128AndNot v128_andnot;
            V128 /*simd128*/ "v128.or" V128Or v128_or;
            V128 /*simd128*/ "v128.xor" V128Xor v128_xor;
            V128 /*simd128*/ "v128.bitselect" V128Bitselect v128_bitselect;
            V128 /*simd128*/ "v128.any_true" V128AnyTrue v128_any_true;
            V128 /*simd128*/ "f32x4.demote_f64x2_zero" F32x4DemoteF64x2Zero f32x4_demote_f64x2_zero;
            V128 /*simd128*/ "f64x2.promote_low_f32x4" F64x2PromoteLowF32x4 f64x2_promote_low_f32x4;
            V128 /*simd128*/ "i8x16.abs" I8x16Abs i8x16_abs;
            V128 /*simd128*/ "i8x16.neg" I8x16Neg i8x16_neg;
            V128 /*simd128*/ "i8x16.popcnt" I8x16Popcnt i8x16_popcnt;
            V128 /*simd128*/ "i8x16.all_true" I8x16AllTrue i8x16_all_true;
            V128 /*simd128*/ "i8x16.bitmask" I8x16Bitmask i8x16_bitmask;
            V128 /*simd128*/ "i8x16.narrow_i16x8_s" I8x16NarrowI16x8S i8x16_narrow_i16x8_s;
            V128 /*simd128*/ "i8x16.narrow_i16x8_u" I8x16NarrowI16x8U i8x16_narrow_i16x8_u;
            V128 /*simd128*/ "f32x4.ceil" F32x4Ceil f32x4_ceil;
            V128 /*simd128*/ "f32x4.floor" F32x4Floor f32x4_floor;
            V128 /*simd128*/ "f32x4.trunc" F32x4Trunc f32x4_trunc;
            V128 /*simd128*/ "f32x4.nearest" F32x4Nearest f32x4_nearest;
            V128 /*simd128*/ "i8x16.shl" I8x16Shl i8x16_shl;
            V128 /*simd128*/ "i8x16.shr_s" I8x16ShrS i8x16_shr_s;
            V128 /*simd128*/ "i8x16.shr_u" I8x16ShrU i8x16_shr_u;
            V128 /*simd128*/ "i8x16.add" I8x16Add i8x16_add;
            V128 /*simd128*/ "i8x16.add_sat_s" I8x16AddSatS i8x16_add_sat_s;
            V128 /*simd128*/ "i8x16.add_sat_u" I8x16AddSatU i8x16_add_sat_u;
            V128 /*simd128*/ "i8x16.sub" I8x16Sub i8x16_sub;
            V128 /*simd128*/ "i8x16.sub_sat_s" I8x16SubSatS i8x16_sub_sat_s;
            V128 /*simd128*/ "i8x16.sub_sat_u" I8x16SubSatU i8x16_sub_sat_u;
            V128 /*simd128*/ "f64x2.ceil" F64x2Ceil f64x2_ceil;
            V128 /*simd128*/ "f64x2.floor" F64x2Floor f64x2_floor;
            V128 /*simd128*/ "i8x16.min_s" I8x16MinS i8x16_min_s;
            V128 /*simd128*/ "i8x16.min_u" I8x16MinU i8x16_min_u;
            V128 /*simd128*/ "i8x16.max_s" I8x16MaxS i8x16_max_s;
            V128 /*simd128*/ "i8x16.max_u" I8x16MaxU i8x16_max_u;
            V128 /*simd128*/ "f64x2.trunc" F64x2Trunc f64x2_trunc;
            V128 /*simd128*/ "i8x16.avgr_u" I8x16AvgrU i8x16_avgr_u;
            V128 /*simd128*/ "i16x8.extadd_pairwise_i8x16_s" I16x8ExtaddPairwiseI8x16S i16x8_extadd_pairwise_i8x16_s;
            V128 /*simd128*/ "i16x8.extadd_pairwise_i8x16_u" I16x8ExtaddPairwiseI8x16U i16x8_extadd_pairwise_i8x16_u;
            V128 /*simd128*/ "i32x4.extadd_pairwise_i16x8_s" I32x4ExtaddPairwiseI16x8S i32x4_extadd_pairwise_i16x8_s;
            V128 /*simd128*/ "i32x4.extadd_pairwise_i16x8_u" I32x4ExtaddPairwiseI16x8U i32x4_extadd_pairwise_i16x8_u;
            V128 /*simd128*/ "i16x8.abs" I16x8Abs i16x8_abs;
            V128 /*simd128*/ "i16x8.neg" I16x8Neg i16x8_neg;
            V128 /*simd128*/ "i16x8.q15mulr_sat_s" I16x8Q15mulrSatS i16x8_q15mulr_sat_s;
            V128 /*simd128*/ "i16x8.all_true" I16x8AllTrue i16x8_all_true;
            V128 /*simd128*/ "i16x8.bitmask" I16x8Bitmask i16x8_bitmask;
            V128 /*simd128*/ "i16x8.narrow_i32x4_s" I16x8NarrowI32x4S i16x8_narrow_i32x4_s;
            V128 /*simd128*/ "i16x8.narrow_i32x4_u" I16x8NarrowI32x4U i16x8_narrow_i32x4_u;
            V128 /*simd128*/ "i16x8.extend_low_i8x16_s" I16x8ExtendLowI8x16S i16x8_extend_low_i8x16_s;
            V128 /*simd128*/ "i16x8.extend_high_i8x16_s" I16x8ExtendHighI8x16S i16x8_extend_high_i8x16_s;
            V128 /*simd128*/ "i16x8.extend_low_i8x16_u" I16x8ExtendLowI8x16U i16x8_extend_low_i8x16_u;
            V128 /*simd128*/ "i16x8.extend_high_i8x16_u" I16x8ExtendHighI8x16U i16x8_extend_high_i8x16_u;
            V128 /*simd128*/ "i16x8.shl" I16x8Shl i16x8_shl;
            V128 /*simd128*/ "i16x8.shr_s" I16x8ShrS i16x8_shr_s;
            V128 /*simd128*/ "i16x8.shr_u" I16x8ShrU i16x8_shr_u;
            V128 /*simd128*/ "i16x8.add" I16x8Add i16x8_add;
            V128 /*simd128*/ "i16x8.add_sat_s" I16x8AddSatS i16x8_add_sat_s;
            V128 /*simd128*/ "i16x8.add_sat_u" I16x8AddSatU i16x8_add_sat_u;
            V128 /*simd128*/ "i16x8.sub" I16x8Sub i16x8_sub;
            V128 /*simd128*/ "i16x8.sub_sat_s" I16x8SubSatS i16x8_sub_sat_s;
            V128 /*simd128*/ "i16x8.sub_sat_u" I16x8SubSatU i16x8_sub_sat_u;
            V128 /*simd128*/ "f64x2.nearest" F64x2Nearest f64x2_nearest;
            V128 /*simd128*/ "i16x8.mul" I16x8Mul i16x8_mul;
            V128 /*simd128*/ "i16x8.min_s" I16x8MinS i16x8_min_s;
            V128 /*simd128*/ "i16x8.min_u" I16x8MinU i16x8_min_u;
            V128 /*simd128*/ "i16x8.max_s" I16x8MaxS i16x8_max_s;
            V128 /*simd128*/ "i16x8.max_u" I16x8MaxU i16x8_max_u;
            V128 /*simd128*/ "i16x8.avgr_u" I16x8AvgrU i16x8_avgr_u;
            V128 /*simd128*/ "i16x8.extmul_low_i8x16_s" I16x8ExtmulLowI8x16S i16x8_extmul_low_i8x16_s;
            V128 /*simd128*/ "i16x8.extmul_high_i8x16_s" I16x8ExtmulHighI8x16S i16x8_extmul_high_i8x16_s;
            V128 /*simd128*/ "i16x8.extmul_low_i8x16_u" I16x8ExtmulLowI8x16U i16x8_extmul_low_i8x16_u;
            V128 /*simd128*/ "i16x8.extmul_high_i8x16_u" I16x8ExtmulHighI8x16U i16x8_extmul_high_i8x16_u;
            V128 /*simd128*/ "i32x4.abs" I32x4Abs i32x4_abs;
            V128 /*simd128*/ "i32x4.neg" I32x4Neg i32x4_neg;
            V128 /*simd128*/ "i32x4.all_true" I32x4AllTrue i32x4_all_true;
            V128 /*simd128*/ "i32x4.bitmask" I32x4Bitmask i32x4_bitmask;
            V128 /*simd128*/ "i32x4.extend_low_i16x8_s" I32x4ExtendLowI16x8S i32x4_extend_low_i16x8_s;
            V128 /*simd128*/ "i32x4.extend_high_i16x8_s" I32x4ExtendHighI16x8S i32x4_extend_high_i16x8_s;
            V128 /*simd128*/ "i32x4.extend_low_i16x8_u" I32x4ExtendLowI16x8U i32x4_extend_low_i16x8_u;
            V128 /*simd128*/ "i32x4.extend_high_i16x8_u" I32x4ExtendHighI16x8U i32x4_extend_high_i16x8_u;
            V128 /*simd128*/ "i32x4.shl" I32x4Shl i32x4_shl;
            V128 /*simd128*/ "i32x4.shr_s" I32x4ShrS i32x4_shr_s;
            V128 /*simd128*/ "i32x4.shr_u" I32x4ShrU i32x4_shr_u;
            V128 /*simd128*/ "i32x4.add" I32x4Add i32x4_add;
            V128 /*simd128*/ "i32x4.sub" I32x4Sub i32x4_sub;
            V128 /*simd128*/ "i32x4.mul" I32x4Mul i32x4_mul;
            V128 /*simd128*/ "i32x4.min_s" I32x4MinS i32x4_min_s;
            V128 /*simd128*/ "i32x4.min_u" I32x4MinU i32x4_min_u;
            V128 /*simd128*/ "i32x4.max_s" I32x4MaxS i32x4_max_s;
            V128 /*simd128*/ "i32x4.max_u" I32x4MaxU i32x4_max_u;
            V128 /*simd128*/ "i32x4.dot_i16x8_s" I32x4DotI16x8S i32x4_dot_i16x8_s;
            V128 /*simd128*/ "i32x4.extmul_low_i16x8_s" I32x4ExtmulLowI16x8S i32x4_extmul_low_i16x8_s;
            V128 /*simd128*/ "i32x4.extmul_high_i16x8_s" I32x4ExtmulHighI16x8S i32x4_extmul_high_i16x8_s;
            V128 /*simd128*/ "i32x4.extmul_low_i16x8_u" I32x4ExtmulLowI16x8U i32x4_extmul_low_i16x8_u;
            V128 /*simd128*/ "i32x4.extmul_high_i16x8_u" I32x4ExtmulHighI16x8U i32x4_extmul_high_i16x8_u;
            V128 /*simd128*/ "i64x2.abs" I64x2Abs i64x2_abs;
            V128 /*simd128*/ "i64x2.neg" I64x2Neg i64x2_neg;
            V128 /*simd128*/ "i64x2.all_true" I64x2AllTrue i64x2_all_true;
            V128 /*simd128*/ "i64x2.bitmask" I64x2Bitmask i64x2_bitmask;
            V128 /*simd128*/ "i64x2.extend_low_i32x4_s" I64x2ExtendLowI32x4S i64x2_extend_low_i32x4_s;
            V128 /*simd128*/ "i64x2.extend_high_i32x4_s" I64x2ExtendHighI32x4S i64x2_extend_high_i32x4_s;
            V128 /*simd128*/ "i64x2.extend_low_i32x4_u" I64x2ExtendLowI32x4U i64x2_extend_low_i32x4_u;
            V128 /*simd128*/ "i64x2.extend_high_i32x4_u" I64x2ExtendHighI32x4U i64x2_extend_high_i32x4_u;
            V128 /*simd128*/ "i64x2.shl" I64x2Shl i64x2_shl;
            V128 /*simd128*/ "i64x2.shr_s" I64x2ShrS i64x2_shr_s;
            V128 /*simd128*/ "i64x2.shr_u" I64x2ShrU i64x2_shr_u;
            V128 /*simd128*/ "i64x2.add" I64x2Add i64x2_add;
            V128 /*simd128*/ "i64x2.sub" I64x2Sub i64x2_sub;
            V128 /*simd128*/ "i64x2.mul" I64x2Mul i64x2_mul;
            V128 /*simd128*/ "i64x2.eq" I64x2Eq i64x2_eq;
            V128 /*simd128*/ "i64x2.ne" I64x2Ne i64x2_ne;
            V128 /*simd128*/ "i64x2.lt_s" I64x2LtS i64x2_lt_s;
            V128 /*simd128*/ "i64x2.gt_s" I64x2GtS i64x2_gt_s;
            V128 /*simd128*/ "i64x2.le_s" I64x2LeS i64x2_le_s;
            V128 /*simd128*/ "i64x2.ge_s" I64x2GeS i64x2_ge_s;
            V128 /*simd128*/ "i64x2.extmul_low_i32x4_s" I64x2ExtmulLowI32x4S i64x2_extmul_low_i32x4_s;
            V128 /*simd128*/ "i64x2.extmul_high_i32x4_s" I64x2ExtmulHighI32x4S i64x2_extmul_high_i32x4_s;
            V128 /*simd128*/ "i64x2.extmul_low_i32x4_u" I64x2ExtmulLowI32x4U i64x2_extmul_low_i32x4_u;
            V128 /*simd128*/ "i64x2.extmul_high_i32x4_u" I64x2ExtmulHighI32x4U i64x2_extmul_high_i32x4_u;
            V128 /*simd128*/ "f32x4.abs" F32x4Abs f32x4_abs;
            V128 /*simd128*/ "f32x4.neg" F32x4Neg f32x4_neg;
            V128 /*simd128*/ "f32x4.sqrt" F32x4Sqrt f32x4_sqrt;
            V128 /*simd128*/ "f32x4.add" F32x4Add f32x4_add;
            V128 /*simd128*/ "f32x4.sub" F32x4Sub f32x4_sub;
            V128 /*simd128*/ "f32x4.mul" F32x4Mul f32x4_mul;
            V128 /*simd128*/ "f32x4.div" F32x4Div f32x4_div;
            V128 /*simd128*/ "f32x4.min" F32x4Min f32x4_min;
            V128 /*simd128*/ "f32x4.max" F32x4Max f32x4_max;
            V128 /*simd128*/ "f32x4.pmin" F32x4Pmin f32x4_pmin;
            V128 /*simd128*/ "f32x4.pmax" F32x4Pmax f32x4_pmax;
            V128 /*simd128*/ "f64x2.abs" F64x2Abs f64x2_abs;
            V128 /*simd128*/ "f64x2.neg" F64x2Neg f64x2_neg;
            V128 /*simd128*/ "f64x2.sqrt" F64x2Sqrt f64x2_sqrt;
            V128 /*simd128*/ "f64x2.add" F64x2Add f64x2_add;
            V128 /*simd128*/ "f64x2.sub" F64x2Sub f64x2_sub;
            V128 /*simd128*/ "f64x2.mul" F64x2Mul f64x2_mul;
            V128 /*simd128*/ "f64x2.div" F64x2Div f64x2_div;
            V128 /*simd128*/ "f64x2.min" F64x2Min f64x2_min;
            V128 /*simd128*/ "f64x2.max" F64x2Max f64x2_max;
            V128 /*simd128*/ "f64x2.pmin" F64x2Pmin f64x2_pmin;
            V128 /*simd128*/ "f64x2.pmax" F64x2Pmax f64x2_pmax;
            V128 /*simd128*/ "i32x4.trunc_sat_f32x4_s" I32x4TruncSatF32x4S i32x4_trunc_sat_f32x4_s;
            V128 /*simd128*/ "i32x4.trunc_sat_f32x4_u" I32x4TruncSatF32x4U i32x4_trunc_sat_f32x4_u;
            V128 /*simd128*/ "f32x4.convert_i32x4_s" F32x4ConvertI32x4S f32x4_convert_i32x4_s;
            V128 /*simd128*/ "f32x4.convert_i32x4_u" F32x4ConvertI32x4U f32x4_convert_i32x4_u;
            V128 /*simd128*/ "i32x4.trunc_sat_f64x2_s_zero" I32x4TruncSatF64x2SZero i32x4_trunc_sat_f64x2_s_zero;
            V128 /*simd128*/ "i32x4.trunc_sat_f64x2_u_zero" I32x4TruncSatF64x2UZero i32x4_trunc_sat_f64x2_u_zero;
            V128 /*simd128*/ "f64x2.convert_low_i32x4_s" F64x2ConvertLowI32x4S f64x2_convert_low_i32x4_s;
            V128 /*simd128*/ "f64x2.convert_low_i32x4_u" F64x2ConvertLowI32x4U f64x2_convert_low_i32x4_u;

            // Tail Call, Control

            Byte /*tail_call*/ "return_call" ReturnCall { callee: FuncIdx } return_call;
            Byte /*tail_call*/ "return_call_indirect" ReturnCallIndirect { signature: TypeIdx, table: TableIdx } return_call_indirect;

            // Threads, Memory

            FEPrefixed  /*atomics*/ "memory.atomic.notify" MemoryAtomicNotify { arg: MemArg } memory_atomic_notify;
            FEPrefixed  /*atomics*/ "memory.atomic.wait32" MemoryAtomicWait32 { arg: MemArg } memory_atomic_wait32;
            FEPrefixed  /*atomics*/ "memory.atomic.wait64" MemoryAtomicWait64 { arg: MemArg } memory_atomic_wait64;
            FEPrefixed  /*atomics*/ "i32.atomic.load" I32AtomicLoad { arg: MemArg } i32_atomic_load;
            FEPrefixed  /*atomics*/ "i64.atomic.load" I64AtomicLoad { arg: MemArg } i64_atomic_load;
            FEPrefixed  /*atomics*/ "i32.atomic.load8_u" I32AtomicLoad8U { arg: MemArg } i32_atomic_load8_u;
            FEPrefixed  /*atomics*/ "i32.atomic.load16_u" I32AtomicLoad16U { arg: MemArg } i32_atomic_load16_u;
            FEPrefixed  /*atomics*/ "i64.atomic.load8_u" I64AtomicLoad8U { arg: MemArg } i64_atomic_load8_u;
            FEPrefixed  /*atomics*/ "i64.atomic.load16_u" I64AtomicLoad16U { arg: MemArg } i64_atomic_load16_u;
            FEPrefixed  /*atomics*/ "i64.atomic.load32_u" I64AtomicLoad32U { arg: MemArg } i64_atomic_load32_u;
            FEPrefixed  /*atomics*/ "i32.atomic.store" I32AtomicStore { arg: MemArg } i32_atomic_store;
            FEPrefixed  /*atomics*/ "i64.atomic.store" I64AtomicStore { arg: MemArg } i64_atomic_store;
            FEPrefixed  /*atomics*/ "i32.atomic.store8_u" I32AtomicStore8U { arg: MemArg } i32_atomic_store8_u;
            FEPrefixed  /*atomics*/ "i32.atomic.store16_u" I32AtomicStore16U { arg: MemArg } i32_atomic_store16_u;
            FEPrefixed  /*atomics*/ "i64.atomic.store8_u" I64AtomicStore8U { arg: MemArg } i64_atomic_store8_u;
            FEPrefixed  /*atomics*/ "i64.atomic.store16_u" I64AtomicStore16U { arg: MemArg } i64_atomic_store16_u;
            FEPrefixed  /*atomics*/ "i64.atomic.store32_u" I64AtomicStore32U { arg: MemArg } i64_atomic_store32_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.add" I32AtomicRmwAdd { arg: MemArg } i32_atomic_rmw_add;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.add" I64AtomicRmwAdd { arg: MemArg } i64_atomic_rmw_add;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.add_u" I32AtomicRmw8AddU { arg: MemArg } i32_atomic_rmw8_add_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.add_u" I32AtomicRmw16AddU { arg: MemArg } i32_atomic_rmw16_add_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.add_u" I64AtomicRmw8AddU { arg: MemArg } i64_atomic_rmw8_add_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.add_u" I64AtomicRmw16AddU { arg: MemArg } i64_atomic_rmw16_add_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.add_u" I64AtomicRmw32AddU { arg: MemArg } i64_atomic_rmw32_add_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.sub" I32AtomicRmwSub { arg: MemArg } i32_atomic_rmw_sub;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.sub" I64AtomicRmwSub { arg: MemArg } i64_atomic_rmw_sub;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.sub_u" I32AtomicRmw8SubU { arg: MemArg } i32_atomic_rmw8_sub_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.sub_u" I32AtomicRmw16SubU { arg: MemArg } i32_atomic_rmw16_sub_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.sub_u" I64AtomicRmw8SubU { arg: MemArg } i64_atomic_rmw8_sub_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.sub_u" I64AtomicRmw16SubU { arg: MemArg } i64_atomic_rmw16_sub_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.sub_u" I64AtomicRmw32SubU { arg: MemArg } i64_atomic_rmw32_sub_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.and" I32AtomicRmwAnd { arg: MemArg } i32_atomic_rmw_and;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.and" I64AtomicRmwAnd { arg: MemArg } i64_atomic_rmw_and;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.and_u" I32AtomicRmw8AndU { arg: MemArg } i32_atomic_rmw8_and_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.and_u" I32AtomicRmw16AndU { arg: MemArg } i32_atomic_rmw16_and_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.and_u" I64AtomicRmw8AndU { arg: MemArg } i64_atomic_rmw8_and_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.and_u" I64AtomicRmw16AndU { arg: MemArg } i64_atomic_rmw16_and_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.and_u" I64AtomicRmw32AndU { arg: MemArg } i64_atomic_rmw32_and_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.or" I32AtomicRmwOr { arg: MemArg } i32_atomic_rmw_or;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.or" I64AtomicRmwOr { arg: MemArg } i64_atomic_rmw_or;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.or_u" I32AtomicRmw8OrU { arg: MemArg } i32_atomic_rmw8_or_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.or_u" I32AtomicRmw16OrU { arg: MemArg } i32_atomic_rmw16_or_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.or_u" I64AtomicRmw8OrU { arg: MemArg } i64_atomic_rmw8_or_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.or_u" I64AtomicRmw16OrU { arg: MemArg } i64_atomic_rmw16_or_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.or_u" I64AtomicRmw32OrU { arg: MemArg } i64_atomic_rmw32_or_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.xor" I32AtomicRmwXor { arg: MemArg } i32_atomic_rmw_xor;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.xor" I64AtomicRmwXor { arg: MemArg } i64_atomic_rmw_xor;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.xor_u" I32AtomicRmw8XorU { arg: MemArg } i32_atomic_rmw8_xor_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.xor_u" I32AtomicRmw16XorU { arg: MemArg } i32_atomic_rmw16_xor_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.xor_u" I64AtomicRmw8XorU { arg: MemArg } i64_atomic_rmw8_xor_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.xor_u" I64AtomicRmw16XorU { arg: MemArg } i64_atomic_rmw16_xor_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.xor_u" I64AtomicRmw32XorU { arg: MemArg } i64_atomic_rmw32_xor_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.xchg" I32AtomicRmwXchg { arg: MemArg } i32_atomic_rmw_xchg;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.xchg" I64AtomicRmwXchg { arg: MemArg } i64_atomic_rmw_xchg;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.xchg_u" I32AtomicRmw8XchgU { arg: MemArg } i32_atomic_rmw8_xchg_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.xchg_u" I32AtomicRmw16XchgU { arg: MemArg } i32_atomic_rmw16_xchg_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.xchg_u" I64AtomicRmw8XchgU { arg: MemArg } i64_atomic_rmw8_xchg_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.xchg_u" I64AtomicRmw16XchgU { arg: MemArg } i64_atomic_rmw16_xchg_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.xchg_u" I64AtomicRmw32XchgU { arg: MemArg } i64_atomic_rmw32_xchg_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw.cmpxchg" I32AtomicRmwCmpxchg { arg: MemArg } i32_atomic_rmw_cmpxchg;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw.cmpxchg" I64AtomicRmwCmpxchg { arg: MemArg } i64_atomic_rmw_cmpxchg;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw8.cmpxchg_u" I32AtomicRmw8CmpxchgU { arg: MemArg } i32_atomic_rmw8_cmpxchg_u;
            FEPrefixed  /*atomics*/ "i32.atomic.rmw16.cmpxchg_u" I32AtomicRmw16CmpxchgU { arg: MemArg } i32_atomic_rmw16_cmpxchg_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw8.cmpxchg_u" I64AtomicRmw8CmpxchgU { arg: MemArg } i64_atomic_rmw8_cmpxchg_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw16.cmpxchg_u" I64AtomicRmw16CmpxchgU { arg: MemArg } i64_atomic_rmw16_cmpxchg_u;
            FEPrefixed  /*atomics*/ "i64.atomic.rmw32.cmpxchg_u" I64AtomicRmw32CmpxchgU { arg: MemArg } i64_atomic_rmw32_cmpxchg_u;

            // Exception Handling, Control

            Byte /*exception_handling*/ "try" Try { block_type: BlockType } r#try;
            Byte /*exception_handling*/ "catch" Catch { exception: TagIdx } r#catch;
            Byte /*exception_handling*/ "throw" Throw { exception: TagIdx } r#throw;
            Byte /*exception_handling*/ "rethrow" Rethrow { handler: LabelIdx } rethrow;
            Byte /*exception_handling*/ "delegate" Delegate { handler: LabelIdx } delegate;
            Byte /*exception_handling*/ "catch_all" CatchAll catch_all;

            // Relaxed SIMD, Vector

            V128 /*relaxed_simd*/ "i8x16.relaxed_swizzle" I8x16RelaxedSwizzle i8x16_relaxed_swizzle;
            V128 /*relaxed_simd*/ "i32x4.relaxed_trunc_f32x4_s" I32x4RelaxedTruncF32x4S i32x4_relaxed_trunc_f32x4_s;
            V128 /*relaxed_simd*/ "i32x4.relaxed_trunc_f32x4_u" I32x4RelaxedTruncF32x4U i32x4_relaxed_trunc_f32x4_u;
            V128 /*relaxed_simd*/ "i32x4.relaxed_trunc_f64x2_s_zero" I32x4RelaxedTruncF64x2SZero i32x4_relaxed_trunc_f64x2_s_zero;
            V128 /*relaxed_simd*/ "i32x4.relaxed_trunc_f64x2_u_zero" I32x4RelaxedTruncF64x2UZero i32x4_relaxed_trunc_f64x2_u_zero;
            V128 /*relaxed_simd*/ "f32x4.relaxed_madd" F32x4RelaxedMadd f32x4_relaxed_madd;
            V128 /*relaxed_simd*/ "f32x4.relaxed_nmadd" F32x4RelaxedNmadd f32x4_relaxed_nmadd;
            V128 /*relaxed_simd*/ "f64x2.relaxed_madd" F64x2RelaxedMadd f64x2_relaxed_madd;
            V128 /*relaxed_simd*/ "f64x2.relaxed_nmadd" F64x2RelaxedNmadd f64x2_relaxed_nmadd;
            V128 /*relaxed_simd*/ "i8x16.relaxed_laneselect" I8x16RelaxedLaneselect i8x16_relaxed_laneselect;
            V128 /*relaxed_simd*/ "i16x8.relaxed_laneselect" I16x8RelaxedLaneselect i16x8_relaxed_laneselect;
            V128 /*relaxed_simd*/ "i32x4.relaxed_laneselect" I32x4RelaxedLaneselect i32x4_relaxed_laneselect;
            V128 /*relaxed_simd*/ "i64x2.relaxed_laneselect" I64x2RelaxedLaneselect i64x2_relaxed_laneselect;
            V128 /*relaxed_simd*/ "f32x4.relaxed_min" F32x4RelaxedMin f32x4_relaxed_min;
            V128 /*relaxed_simd*/ "f32x4.relaxed_max" F32x4RelaxedMax f32x4_relaxed_max;
            V128 /*relaxed_simd*/ "f64x2.relaxed_min" F64x2RelaxedMin f64x2_relaxed_min;
            V128 /*relaxed_simd*/ "f64x2.relaxed_max" F64x2RelaxedMax f64x2_relaxed_max;
            V128 /*relaxed_simd*/ "i16x8.relaxed_q15mulr_s" I16x8RelaxedQ15mulrS i16x8_relaxed_q15mulr_s;
            V128 /*relaxed_simd*/ "i16x8.relaxed_dot_i8x16_i7x16_s" I16x8RelaxedDotI8x16I7x16S i16x8_relaxed_dot_i8x16_i7x16_s;
            V128 /*relaxed_simd*/ "i32x4.relaxed_dot_i8x16_i7x16_add_s" I32x4RelaxedDotI8x16I7x16AddS i32x4_relaxed_dot_i8x16_i7x16_add_s;
        }
    };
    ($called_macro:ident) => {
        crate::isa::instr_definitions::all!($called_macro @ ignore_field_type);
    }
}

pub(in crate::isa) use all;
