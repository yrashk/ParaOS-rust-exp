use crate::x86_64::Error;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec;
use alloc::vec::Vec;
use iced_x86::code_asm::{
    eax, ebx, ptr, r10, r11, r8, r9, rax, rbp, rbx, rcx, rdi, rdx, rsi, AsmRegister64,
    CodeAssembler, CodeLabel,
};
use wasmparser_nostd::{FuncType, Operator, Type};

pub(crate) fn handle_instruction(
    assembler: &mut CodeAssembler,
    got: &mut BTreeMap<u32, CodeLabel>,
    ils: &mut BTreeMap<u32, CodeLabel>,
    function_typedefs: &mut BTreeMap<u32, FuncType>,
    function_types: &mut BTreeMap<u32, u32>,
    locals: &Vec<u32>,
    op: Operator,
) -> Result<(), Error> {
    match op {
        Operator::I64Const { value } => {
            assembler.mov(rax, value)?;
            assembler.push(rax)?;
        }
        Operator::I64Add => {
            assembler.pop(rax)?;
            assembler.pop(rbx)?;
            assembler.add(rax, rbx)?;
            assembler.push(rax)?;
        }
        Operator::I32Add => {
            assembler.pop(eax)?;
            assembler.pop(ebx)?;
            assembler.add(eax, ebx)?;
            assembler.push(eax)?;
        }
        Operator::I64Sub => {
            assembler.pop(rax)?;
            assembler.pop(rbx)?;
            assembler.sub(rax, rbx)?;
            assembler.push(rax)?;
        }
        Operator::I32Sub => {
            assembler.pop(eax)?;
            assembler.pop(ebx)?;
            assembler.sub(eax, ebx)?;
            assembler.push(eax)?;
        }
        Operator::Call { function_index } => {
            let called_function_type = function_types
                .get(&function_index)
                .and_then(|i| function_types.get(&i))
                .and_then(|t| function_typedefs.get(t))
                .cloned()
                .unwrap();
            let mut integer_order: VecDeque<AsmRegister64> = vec![rdi, rsi, rdx, rcx, r8, r9]
                .drain(0..called_function_type.params.len())
                .collect();
            for param in called_function_type.params.iter() {
                match param {
                    Type::I64 | Type::I32 => match integer_order.pop_back() {
                        Some(reg) => assembler.pop(reg)?,
                        None => todo!(),
                    },
                    _ => todo!(),
                }
            }
            match got.get(&function_index) {
                None => {
                    if let Some(import_label) = ils.get(&function_index) {
                        assembler.mov(r10, ptr(*import_label))?;
                        assembler.push(r11)?;
                        assembler.call(r10)?;
                        assembler.pop(r11)?;
                    }
                }
                Some(label) => {
                    assembler.push(r11)?;
                    assembler.call(*label)?;
                    assembler.pop(r11)?;
                }
            }
            let mut integer_order = VecDeque::from([rax, rdx]);
            for ret in called_function_type.returns.iter() {
                match ret {
                    Type::I64 | Type::I32 => match integer_order.pop_front() {
                        Some(reg) => assembler.push(reg)?,
                        None => (),
                    },
                    _ => todo!(),
                }
            }
        }
        Operator::Unreachable => todo!(),
        Operator::Nop => assembler.nop()?,
        Operator::Block { .. } => todo!(),
        Operator::Loop { .. } => todo!(),
        Operator::If { .. } => todo!(),
        Operator::Else => todo!(),
        Operator::Try { .. } => todo!(),
        Operator::Catch { .. } => todo!(),
        Operator::Throw { .. } => todo!(),
        Operator::Rethrow { .. } => todo!(),
        Operator::End => {}
        Operator::Br { .. } => todo!(),
        Operator::BrIf { .. } => todo!(),
        Operator::BrTable { .. } => todo!(),
        Operator::Return => todo!(),
        Operator::CallIndirect { .. } => todo!(),
        Operator::ReturnCall { .. } => todo!(),
        Operator::ReturnCallIndirect { .. } => todo!(),
        Operator::Delegate { .. } => todo!(),
        Operator::CatchAll => todo!(),
        Operator::Drop => todo!(),
        Operator::Select => todo!(),
        Operator::TypedSelect { .. } => todo!(),
        Operator::LocalGet { local_index } => match locals.get(local_index as usize) {
            Some(offset) => {
                assembler.mov(rax, ptr(rbp - *offset))?;
                assembler.push(rax)?;
            }
            None => todo!(),
        },
        Operator::LocalSet { local_index } => match locals.get(local_index as usize) {
            Some(offset) => {
                assembler.pop(rax)?;
                assembler.mov(ptr(rbp - *offset), rax)?;
            }
            None => todo!(),
        },
        Operator::LocalTee { .. } => todo!(),
        Operator::GlobalGet { .. } => todo!(),
        Operator::GlobalSet { .. } => todo!(),
        Operator::I32Load { .. } => todo!(),
        Operator::I64Load { .. } => todo!(),
        Operator::F32Load { .. } => todo!(),
        Operator::F64Load { .. } => todo!(),
        Operator::I32Load8S { .. } => todo!(),
        Operator::I32Load8U { .. } => todo!(),
        Operator::I32Load16S { .. } => todo!(),
        Operator::I32Load16U { .. } => todo!(),
        Operator::I64Load8S { .. } => todo!(),
        Operator::I64Load8U { .. } => todo!(),
        Operator::I64Load16S { .. } => todo!(),
        Operator::I64Load16U { .. } => todo!(),
        Operator::I64Load32S { .. } => todo!(),
        Operator::I64Load32U { .. } => todo!(),
        Operator::I32Store { .. } => todo!(),
        Operator::I64Store { .. } => todo!(),
        Operator::F32Store { .. } => todo!(),
        Operator::F64Store { .. } => todo!(),
        Operator::I32Store8 { .. } => todo!(),
        Operator::I32Store16 { .. } => todo!(),
        Operator::I64Store8 { .. } => todo!(),
        Operator::I64Store16 { .. } => todo!(),
        Operator::I64Store32 { .. } => todo!(),
        Operator::MemorySize { .. } => todo!(),
        Operator::MemoryGrow { .. } => todo!(),
        Operator::I32Const { .. } => todo!(),
        Operator::F32Const { .. } => todo!(),
        Operator::F64Const { .. } => todo!(),
        Operator::RefNull { .. } => todo!(),
        Operator::RefIsNull => todo!(),
        Operator::RefFunc { .. } => todo!(),
        Operator::I32Eqz => todo!(),
        Operator::I32Eq => todo!(),
        Operator::I32Ne => todo!(),
        Operator::I32LtS => todo!(),
        Operator::I32LtU => todo!(),
        Operator::I32GtS => todo!(),
        Operator::I32GtU => todo!(),
        Operator::I32LeS => todo!(),
        Operator::I32LeU => todo!(),
        Operator::I32GeS => todo!(),
        Operator::I32GeU => todo!(),
        Operator::I64Eqz => todo!(),
        Operator::I64Eq => todo!(),
        Operator::I64Ne => todo!(),
        Operator::I64LtS => todo!(),
        Operator::I64LtU => todo!(),
        Operator::I64GtS => todo!(),
        Operator::I64GtU => todo!(),
        Operator::I64LeS => todo!(),
        Operator::I64LeU => todo!(),
        Operator::I64GeS => todo!(),
        Operator::I64GeU => todo!(),
        Operator::F32Eq => todo!(),
        Operator::F32Ne => todo!(),
        Operator::F32Lt => todo!(),
        Operator::F32Gt => todo!(),
        Operator::F32Le => todo!(),
        Operator::F32Ge => todo!(),
        Operator::F64Eq => todo!(),
        Operator::F64Ne => todo!(),
        Operator::F64Lt => todo!(),
        Operator::F64Gt => todo!(),
        Operator::F64Le => todo!(),
        Operator::F64Ge => todo!(),
        Operator::I32Clz => todo!(),
        Operator::I32Ctz => todo!(),
        Operator::I32Popcnt => todo!(),
        Operator::I32Mul => todo!(),
        Operator::I32DivS => todo!(),
        Operator::I32DivU => todo!(),
        Operator::I32RemS => todo!(),
        Operator::I32RemU => todo!(),
        Operator::I32And => todo!(),
        Operator::I32Or => todo!(),
        Operator::I32Xor => todo!(),
        Operator::I32Shl => todo!(),
        Operator::I32ShrS => todo!(),
        Operator::I32ShrU => todo!(),
        Operator::I32Rotl => todo!(),
        Operator::I32Rotr => todo!(),
        Operator::I64Clz => todo!(),
        Operator::I64Ctz => todo!(),
        Operator::I64Popcnt => todo!(),
        Operator::I64Mul => todo!(),
        Operator::I64DivS => todo!(),
        Operator::I64DivU => todo!(),
        Operator::I64RemS => todo!(),
        Operator::I64RemU => todo!(),
        Operator::I64And => todo!(),
        Operator::I64Or => todo!(),
        Operator::I64Xor => todo!(),
        Operator::I64Shl => todo!(),
        Operator::I64ShrS => todo!(),
        Operator::I64ShrU => todo!(),
        Operator::I64Rotl => todo!(),
        Operator::I64Rotr => todo!(),
        Operator::F32Abs => todo!(),
        Operator::F32Neg => todo!(),
        Operator::F32Ceil => todo!(),
        Operator::F32Floor => todo!(),
        Operator::F32Trunc => todo!(),
        Operator::F32Nearest => todo!(),
        Operator::F32Sqrt => todo!(),
        Operator::F32Add => todo!(),
        Operator::F32Sub => todo!(),
        Operator::F32Mul => todo!(),
        Operator::F32Div => todo!(),
        Operator::F32Min => todo!(),
        Operator::F32Max => todo!(),
        Operator::F32Copysign => todo!(),
        Operator::F64Abs => todo!(),
        Operator::F64Neg => todo!(),
        Operator::F64Ceil => todo!(),
        Operator::F64Floor => todo!(),
        Operator::F64Trunc => todo!(),
        Operator::F64Nearest => todo!(),
        Operator::F64Sqrt => todo!(),
        Operator::F64Add => todo!(),
        Operator::F64Sub => todo!(),
        Operator::F64Mul => todo!(),
        Operator::F64Div => todo!(),
        Operator::F64Min => todo!(),
        Operator::F64Max => todo!(),
        Operator::F64Copysign => todo!(),
        Operator::I32WrapI64 => todo!(),
        Operator::I32TruncF32S => todo!(),
        Operator::I32TruncF32U => todo!(),
        Operator::I32TruncF64S => todo!(),
        Operator::I32TruncF64U => todo!(),
        Operator::I64ExtendI32S => todo!(),
        Operator::I64ExtendI32U => todo!(),
        Operator::I64TruncF32S => todo!(),
        Operator::I64TruncF32U => todo!(),
        Operator::I64TruncF64S => todo!(),
        Operator::I64TruncF64U => todo!(),
        Operator::F32ConvertI32S => todo!(),
        Operator::F32ConvertI32U => todo!(),
        Operator::F32ConvertI64S => todo!(),
        Operator::F32ConvertI64U => todo!(),
        Operator::F32DemoteF64 => todo!(),
        Operator::F64ConvertI32S => todo!(),
        Operator::F64ConvertI32U => todo!(),
        Operator::F64ConvertI64S => todo!(),
        Operator::F64ConvertI64U => todo!(),
        Operator::F64PromoteF32 => todo!(),
        Operator::I32ReinterpretF32 => todo!(),
        Operator::I64ReinterpretF64 => todo!(),
        Operator::F32ReinterpretI32 => todo!(),
        Operator::F64ReinterpretI64 => todo!(),
        Operator::I32Extend8S => todo!(),
        Operator::I32Extend16S => todo!(),
        Operator::I64Extend8S => todo!(),
        Operator::I64Extend16S => todo!(),
        Operator::I64Extend32S => todo!(),
        Operator::I32TruncSatF32S => todo!(),
        Operator::I32TruncSatF32U => todo!(),
        Operator::I32TruncSatF64S => todo!(),
        Operator::I32TruncSatF64U => todo!(),
        Operator::I64TruncSatF32S => todo!(),
        Operator::I64TruncSatF32U => todo!(),
        Operator::I64TruncSatF64S => todo!(),
        Operator::I64TruncSatF64U => todo!(),
        Operator::MemoryInit { .. } => todo!(),
        Operator::DataDrop { .. } => todo!(),
        Operator::MemoryCopy { .. } => todo!(),
        Operator::MemoryFill { .. } => todo!(),
        Operator::TableInit { .. } => todo!(),
        Operator::ElemDrop { .. } => todo!(),
        Operator::TableCopy { .. } => todo!(),
        Operator::TableFill { .. } => todo!(),
        Operator::TableGet { .. } => todo!(),
        Operator::TableSet { .. } => todo!(),
        Operator::TableGrow { .. } => todo!(),
        Operator::TableSize { .. } => todo!(),
        Operator::MemoryAtomicNotify { .. } => todo!(),
        Operator::MemoryAtomicWait32 { .. } => todo!(),
        Operator::MemoryAtomicWait64 { .. } => todo!(),
        Operator::AtomicFence { .. } => todo!(),
        Operator::I32AtomicLoad { .. } => todo!(),
        Operator::I64AtomicLoad { .. } => todo!(),
        Operator::I32AtomicLoad8U { .. } => todo!(),
        Operator::I32AtomicLoad16U { .. } => todo!(),
        Operator::I64AtomicLoad8U { .. } => todo!(),
        Operator::I64AtomicLoad16U { .. } => todo!(),
        Operator::I64AtomicLoad32U { .. } => todo!(),
        Operator::I32AtomicStore { .. } => todo!(),
        Operator::I64AtomicStore { .. } => todo!(),
        Operator::I32AtomicStore8 { .. } => todo!(),
        Operator::I32AtomicStore16 { .. } => todo!(),
        Operator::I64AtomicStore8 { .. } => todo!(),
        Operator::I64AtomicStore16 { .. } => todo!(),
        Operator::I64AtomicStore32 { .. } => todo!(),
        Operator::I32AtomicRmwAdd { .. } => todo!(),
        Operator::I64AtomicRmwAdd { .. } => todo!(),
        Operator::I32AtomicRmw8AddU { .. } => todo!(),
        Operator::I32AtomicRmw16AddU { .. } => todo!(),
        Operator::I64AtomicRmw8AddU { .. } => todo!(),
        Operator::I64AtomicRmw16AddU { .. } => todo!(),
        Operator::I64AtomicRmw32AddU { .. } => todo!(),
        Operator::I32AtomicRmwSub { .. } => todo!(),
        Operator::I64AtomicRmwSub { .. } => todo!(),
        Operator::I32AtomicRmw8SubU { .. } => todo!(),
        Operator::I32AtomicRmw16SubU { .. } => todo!(),
        Operator::I64AtomicRmw8SubU { .. } => todo!(),
        Operator::I64AtomicRmw16SubU { .. } => todo!(),
        Operator::I64AtomicRmw32SubU { .. } => todo!(),
        Operator::I32AtomicRmwAnd { .. } => todo!(),
        Operator::I64AtomicRmwAnd { .. } => todo!(),
        Operator::I32AtomicRmw8AndU { .. } => todo!(),
        Operator::I32AtomicRmw16AndU { .. } => todo!(),
        Operator::I64AtomicRmw8AndU { .. } => todo!(),
        Operator::I64AtomicRmw16AndU { .. } => todo!(),
        Operator::I64AtomicRmw32AndU { .. } => todo!(),
        Operator::I32AtomicRmwOr { .. } => todo!(),
        Operator::I64AtomicRmwOr { .. } => todo!(),
        Operator::I32AtomicRmw8OrU { .. } => todo!(),
        Operator::I32AtomicRmw16OrU { .. } => todo!(),
        Operator::I64AtomicRmw8OrU { .. } => todo!(),
        Operator::I64AtomicRmw16OrU { .. } => todo!(),
        Operator::I64AtomicRmw32OrU { .. } => todo!(),
        Operator::I32AtomicRmwXor { .. } => todo!(),
        Operator::I64AtomicRmwXor { .. } => todo!(),
        Operator::I32AtomicRmw8XorU { .. } => todo!(),
        Operator::I32AtomicRmw16XorU { .. } => todo!(),
        Operator::I64AtomicRmw8XorU { .. } => todo!(),
        Operator::I64AtomicRmw16XorU { .. } => todo!(),
        Operator::I64AtomicRmw32XorU { .. } => todo!(),
        Operator::I32AtomicRmwXchg { .. } => todo!(),
        Operator::I64AtomicRmwXchg { .. } => todo!(),
        Operator::I32AtomicRmw8XchgU { .. } => todo!(),
        Operator::I32AtomicRmw16XchgU { .. } => todo!(),
        Operator::I64AtomicRmw8XchgU { .. } => todo!(),
        Operator::I64AtomicRmw16XchgU { .. } => todo!(),
        Operator::I64AtomicRmw32XchgU { .. } => todo!(),
        Operator::I32AtomicRmwCmpxchg { .. } => todo!(),
        Operator::I64AtomicRmwCmpxchg { .. } => todo!(),
        Operator::I32AtomicRmw8CmpxchgU { .. } => todo!(),
        Operator::I32AtomicRmw16CmpxchgU { .. } => todo!(),
        Operator::I64AtomicRmw8CmpxchgU { .. } => todo!(),
        Operator::I64AtomicRmw16CmpxchgU { .. } => todo!(),
        Operator::I64AtomicRmw32CmpxchgU { .. } => todo!(),
        Operator::V128Load { .. } => todo!(),
        Operator::V128Load8x8S { .. } => todo!(),
        Operator::V128Load8x8U { .. } => todo!(),
        Operator::V128Load16x4S { .. } => todo!(),
        Operator::V128Load16x4U { .. } => todo!(),
        Operator::V128Load32x2S { .. } => todo!(),
        Operator::V128Load32x2U { .. } => todo!(),
        Operator::V128Load8Splat { .. } => todo!(),
        Operator::V128Load16Splat { .. } => todo!(),
        Operator::V128Load32Splat { .. } => todo!(),
        Operator::V128Load64Splat { .. } => todo!(),
        Operator::V128Load32Zero { .. } => todo!(),
        Operator::V128Load64Zero { .. } => todo!(),
        Operator::V128Store { .. } => todo!(),
        Operator::V128Load8Lane { .. } => todo!(),
        Operator::V128Load16Lane { .. } => todo!(),
        Operator::V128Load32Lane { .. } => todo!(),
        Operator::V128Load64Lane { .. } => todo!(),
        Operator::V128Store8Lane { .. } => todo!(),
        Operator::V128Store16Lane { .. } => todo!(),
        Operator::V128Store32Lane { .. } => todo!(),
        Operator::V128Store64Lane { .. } => todo!(),
        Operator::V128Const { .. } => todo!(),
        Operator::I8x16Shuffle { .. } => todo!(),
        Operator::I8x16ExtractLaneS { .. } => todo!(),
        Operator::I8x16ExtractLaneU { .. } => todo!(),
        Operator::I8x16ReplaceLane { .. } => todo!(),
        Operator::I16x8ExtractLaneS { .. } => todo!(),
        Operator::I16x8ExtractLaneU { .. } => todo!(),
        Operator::I16x8ReplaceLane { .. } => todo!(),
        Operator::I32x4ExtractLane { .. } => todo!(),
        Operator::I32x4ReplaceLane { .. } => todo!(),
        Operator::I64x2ExtractLane { .. } => todo!(),
        Operator::I64x2ReplaceLane { .. } => todo!(),
        Operator::F32x4ExtractLane { .. } => todo!(),
        Operator::F32x4ReplaceLane { .. } => todo!(),
        Operator::F64x2ExtractLane { .. } => todo!(),
        Operator::F64x2ReplaceLane { .. } => todo!(),
        Operator::I8x16Swizzle => todo!(),
        Operator::I8x16Splat => todo!(),
        Operator::I16x8Splat => todo!(),
        Operator::I32x4Splat => todo!(),
        Operator::I64x2Splat => todo!(),
        Operator::F32x4Splat => todo!(),
        Operator::F64x2Splat => todo!(),
        Operator::I8x16Eq => todo!(),
        Operator::I8x16Ne => todo!(),
        Operator::I8x16LtS => todo!(),
        Operator::I8x16LtU => todo!(),
        Operator::I8x16GtS => todo!(),
        Operator::I8x16GtU => todo!(),
        Operator::I8x16LeS => todo!(),
        Operator::I8x16LeU => todo!(),
        Operator::I8x16GeS => todo!(),
        Operator::I8x16GeU => todo!(),
        Operator::I16x8Eq => todo!(),
        Operator::I16x8Ne => todo!(),
        Operator::I16x8LtS => todo!(),
        Operator::I16x8LtU => todo!(),
        Operator::I16x8GtS => todo!(),
        Operator::I16x8GtU => todo!(),
        Operator::I16x8LeS => todo!(),
        Operator::I16x8LeU => todo!(),
        Operator::I16x8GeS => todo!(),
        Operator::I16x8GeU => todo!(),
        Operator::I32x4Eq => todo!(),
        Operator::I32x4Ne => todo!(),
        Operator::I32x4LtS => todo!(),
        Operator::I32x4LtU => todo!(),
        Operator::I32x4GtS => todo!(),
        Operator::I32x4GtU => todo!(),
        Operator::I32x4LeS => todo!(),
        Operator::I32x4LeU => todo!(),
        Operator::I32x4GeS => todo!(),
        Operator::I32x4GeU => todo!(),
        Operator::I64x2Eq => todo!(),
        Operator::I64x2Ne => todo!(),
        Operator::I64x2LtS => todo!(),
        Operator::I64x2GtS => todo!(),
        Operator::I64x2LeS => todo!(),
        Operator::I64x2GeS => todo!(),
        Operator::F32x4Eq => todo!(),
        Operator::F32x4Ne => todo!(),
        Operator::F32x4Lt => todo!(),
        Operator::F32x4Gt => todo!(),
        Operator::F32x4Le => todo!(),
        Operator::F32x4Ge => todo!(),
        Operator::F64x2Eq => todo!(),
        Operator::F64x2Ne => todo!(),
        Operator::F64x2Lt => todo!(),
        Operator::F64x2Gt => todo!(),
        Operator::F64x2Le => todo!(),
        Operator::F64x2Ge => todo!(),
        Operator::V128Not => todo!(),
        Operator::V128And => todo!(),
        Operator::V128AndNot => todo!(),
        Operator::V128Or => todo!(),
        Operator::V128Xor => todo!(),
        Operator::V128Bitselect => todo!(),
        Operator::V128AnyTrue => todo!(),
        Operator::I8x16Abs => todo!(),
        Operator::I8x16Neg => todo!(),
        Operator::I8x16Popcnt => todo!(),
        Operator::I8x16AllTrue => todo!(),
        Operator::I8x16Bitmask => todo!(),
        Operator::I8x16NarrowI16x8S => todo!(),
        Operator::I8x16NarrowI16x8U => todo!(),
        Operator::I8x16Shl => todo!(),
        Operator::I8x16ShrS => todo!(),
        Operator::I8x16ShrU => todo!(),
        Operator::I8x16Add => todo!(),
        Operator::I8x16AddSatS => todo!(),
        Operator::I8x16AddSatU => todo!(),
        Operator::I8x16Sub => todo!(),
        Operator::I8x16SubSatS => todo!(),
        Operator::I8x16SubSatU => todo!(),
        Operator::I8x16MinS => todo!(),
        Operator::I8x16MinU => todo!(),
        Operator::I8x16MaxS => todo!(),
        Operator::I8x16MaxU => todo!(),
        Operator::I8x16RoundingAverageU => todo!(),
        Operator::I16x8ExtAddPairwiseI8x16S => todo!(),
        Operator::I16x8ExtAddPairwiseI8x16U => todo!(),
        Operator::I16x8Abs => todo!(),
        Operator::I16x8Neg => todo!(),
        Operator::I16x8Q15MulrSatS => todo!(),
        Operator::I16x8AllTrue => todo!(),
        Operator::I16x8Bitmask => todo!(),
        Operator::I16x8NarrowI32x4S => todo!(),
        Operator::I16x8NarrowI32x4U => todo!(),
        Operator::I16x8ExtendLowI8x16S => todo!(),
        Operator::I16x8ExtendHighI8x16S => todo!(),
        Operator::I16x8ExtendLowI8x16U => todo!(),
        Operator::I16x8ExtendHighI8x16U => todo!(),
        Operator::I16x8Shl => todo!(),
        Operator::I16x8ShrS => todo!(),
        Operator::I16x8ShrU => todo!(),
        Operator::I16x8Add => todo!(),
        Operator::I16x8AddSatS => todo!(),
        Operator::I16x8AddSatU => todo!(),
        Operator::I16x8Sub => todo!(),
        Operator::I16x8SubSatS => todo!(),
        Operator::I16x8SubSatU => todo!(),
        Operator::I16x8Mul => todo!(),
        Operator::I16x8MinS => todo!(),
        Operator::I16x8MinU => todo!(),
        Operator::I16x8MaxS => todo!(),
        Operator::I16x8MaxU => todo!(),
        Operator::I16x8RoundingAverageU => todo!(),
        Operator::I16x8ExtMulLowI8x16S => todo!(),
        Operator::I16x8ExtMulHighI8x16S => todo!(),
        Operator::I16x8ExtMulLowI8x16U => todo!(),
        Operator::I16x8ExtMulHighI8x16U => todo!(),
        Operator::I32x4ExtAddPairwiseI16x8S => todo!(),
        Operator::I32x4ExtAddPairwiseI16x8U => todo!(),
        Operator::I32x4Abs => todo!(),
        Operator::I32x4Neg => todo!(),
        Operator::I32x4AllTrue => todo!(),
        Operator::I32x4Bitmask => todo!(),
        Operator::I32x4ExtendLowI16x8S => todo!(),
        Operator::I32x4ExtendHighI16x8S => todo!(),
        Operator::I32x4ExtendLowI16x8U => todo!(),
        Operator::I32x4ExtendHighI16x8U => todo!(),
        Operator::I32x4Shl => todo!(),
        Operator::I32x4ShrS => todo!(),
        Operator::I32x4ShrU => todo!(),
        Operator::I32x4Add => todo!(),
        Operator::I32x4Sub => todo!(),
        Operator::I32x4Mul => todo!(),
        Operator::I32x4MinS => todo!(),
        Operator::I32x4MinU => todo!(),
        Operator::I32x4MaxS => todo!(),
        Operator::I32x4MaxU => todo!(),
        Operator::I32x4DotI16x8S => todo!(),
        Operator::I32x4ExtMulLowI16x8S => todo!(),
        Operator::I32x4ExtMulHighI16x8S => todo!(),
        Operator::I32x4ExtMulLowI16x8U => todo!(),
        Operator::I32x4ExtMulHighI16x8U => todo!(),
        Operator::I64x2Abs => todo!(),
        Operator::I64x2Neg => todo!(),
        Operator::I64x2AllTrue => todo!(),
        Operator::I64x2Bitmask => todo!(),
        Operator::I64x2ExtendLowI32x4S => todo!(),
        Operator::I64x2ExtendHighI32x4S => todo!(),
        Operator::I64x2ExtendLowI32x4U => todo!(),
        Operator::I64x2ExtendHighI32x4U => todo!(),
        Operator::I64x2Shl => todo!(),
        Operator::I64x2ShrS => todo!(),
        Operator::I64x2ShrU => todo!(),
        Operator::I64x2Add => todo!(),
        Operator::I64x2Sub => todo!(),
        Operator::I64x2Mul => todo!(),
        Operator::I64x2ExtMulLowI32x4S => todo!(),
        Operator::I64x2ExtMulHighI32x4S => todo!(),
        Operator::I64x2ExtMulLowI32x4U => todo!(),
        Operator::I64x2ExtMulHighI32x4U => todo!(),
        Operator::F32x4Ceil => todo!(),
        Operator::F32x4Floor => todo!(),
        Operator::F32x4Trunc => todo!(),
        Operator::F32x4Nearest => todo!(),
        Operator::F32x4Abs => todo!(),
        Operator::F32x4Neg => todo!(),
        Operator::F32x4Sqrt => todo!(),
        Operator::F32x4Add => todo!(),
        Operator::F32x4Sub => todo!(),
        Operator::F32x4Mul => todo!(),
        Operator::F32x4Div => todo!(),
        Operator::F32x4Min => todo!(),
        Operator::F32x4Max => todo!(),
        Operator::F32x4PMin => todo!(),
        Operator::F32x4PMax => todo!(),
        Operator::F64x2Ceil => todo!(),
        Operator::F64x2Floor => todo!(),
        Operator::F64x2Trunc => todo!(),
        Operator::F64x2Nearest => todo!(),
        Operator::F64x2Abs => todo!(),
        Operator::F64x2Neg => todo!(),
        Operator::F64x2Sqrt => todo!(),
        Operator::F64x2Add => todo!(),
        Operator::F64x2Sub => todo!(),
        Operator::F64x2Mul => todo!(),
        Operator::F64x2Div => todo!(),
        Operator::F64x2Min => todo!(),
        Operator::F64x2Max => todo!(),
        Operator::F64x2PMin => todo!(),
        Operator::F64x2PMax => todo!(),
        Operator::I32x4TruncSatF32x4S => todo!(),
        Operator::I32x4TruncSatF32x4U => todo!(),
        Operator::F32x4ConvertI32x4S => todo!(),
        Operator::F32x4ConvertI32x4U => todo!(),
        Operator::I32x4TruncSatF64x2SZero => todo!(),
        Operator::I32x4TruncSatF64x2UZero => todo!(),
        Operator::F64x2ConvertLowI32x4S => todo!(),
        Operator::F64x2ConvertLowI32x4U => todo!(),
        Operator::F32x4DemoteF64x2Zero => todo!(),
        Operator::F64x2PromoteLowF32x4 => todo!(),
        Operator::I8x16SwizzleRelaxed => todo!(),
        Operator::I32x4TruncSatF32x4SRelaxed => todo!(),
        Operator::I32x4TruncSatF32x4URelaxed => todo!(),
        Operator::I32x4TruncSatF64x2SZeroRelaxed => todo!(),
        Operator::I32x4TruncSatF64x2UZeroRelaxed => todo!(),
        Operator::F32x4FmaRelaxed => todo!(),
        Operator::F32x4FmsRelaxed => todo!(),
        Operator::F64x2FmaRelaxed => todo!(),
        Operator::F64x2FmsRelaxed => todo!(),
        Operator::I8x16LaneSelect => todo!(),
        Operator::I16x8LaneSelect => todo!(),
        Operator::I32x4LaneSelect => todo!(),
        Operator::I64x2LaneSelect => todo!(),
        Operator::F32x4MinRelaxed => todo!(),
        Operator::F32x4MaxRelaxed => todo!(),
        Operator::F64x2MinRelaxed => todo!(),
        Operator::F64x2MaxRelaxed => todo!(),
    }
    Ok(())
}
