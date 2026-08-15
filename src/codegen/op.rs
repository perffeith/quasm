use crate::lower::ir::IrTy;
use crate::common::op::BinOpKind;
use wasm_encoder::Instruction;

pub fn bin_op(op: BinOpKind, ty: IrTy) -> Instruction<'static> {
    match (op, ty) {
        (BinOpKind::Add, IrTy::I32) => Instruction::I32Add,
        (BinOpKind::Sub, IrTy::I32) => Instruction::I32Sub,
        (BinOpKind::Mul, IrTy::I32) => Instruction::I32Mul,
        (BinOpKind::Div, IrTy::I32) => Instruction::I32DivS,
        (BinOpKind::Lt, IrTy::I32) => Instruction::I32LtS,
        (BinOpKind::Gt, IrTy::I32) => Instruction::I32GtS,
        (BinOpKind::LtEq, IrTy::I32) => Instruction::I32LeS,
        (BinOpKind::GtEq, IrTy::I32) => Instruction::I32GeS,
        (BinOpKind::EqEq, IrTy::I32) => Instruction::I32Eq,
        (BinOpKind::NotEq, IrTy::I32) => Instruction::I32Ne,

        (BinOpKind::Add, IrTy::F32) => Instruction::F32Add,
        (BinOpKind::Sub, IrTy::F32) => Instruction::F32Sub,
        (BinOpKind::Mul, IrTy::F32) => Instruction::F32Mul,
        (BinOpKind::Div, IrTy::F32) => Instruction::F32Div,
        (BinOpKind::Lt, IrTy::F32) => Instruction::F32Lt,
        (BinOpKind::Gt, IrTy::F32) => Instruction::F32Gt,
        (BinOpKind::LtEq, IrTy::F32) => Instruction::F32Le,
        (BinOpKind::GtEq, IrTy::F32) => Instruction::F32Ge,
        (BinOpKind::EqEq, IrTy::F32) => Instruction::F32Eq,
        (BinOpKind::NotEq, IrTy::F32) => Instruction::F32Ne,

        (BinOpKind::And, IrTy::I32) => Instruction::I32And,
        (BinOpKind::Or, IrTy::I32) => Instruction::I32Or,

        (op, ty) => unreachable!("bug: sema let `{op}` on {ty:?} thru to codegen")
    }
}
