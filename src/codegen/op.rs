use crate::lower::ir::IrTy;
use crate::common::op::BinOpKind;
use wasm_encoder::Instruction;

pub fn bin_op(op: BinOpKind, ty: IrTy) -> Instruction<'static> {
    match (op, ty) {
        (BinOpKind::Add, IrTy::I64) => Instruction::I64Add,
        (BinOpKind::Sub, IrTy::I64) => Instruction::I64Sub,
        (BinOpKind::Mul, IrTy::I64) => Instruction::I64Mul,
        (BinOpKind::Div, IrTy::I64) => Instruction::I64DivS,
        (BinOpKind::Lt, IrTy::I64) => Instruction::I64LtS,
        (BinOpKind::Gt, IrTy::I64) => Instruction::I64GtS,
        (BinOpKind::LtEq, IrTy::I64) => Instruction::I64LeS,
        (BinOpKind::GtEq, IrTy::I64) => Instruction::I64GeS,
        (BinOpKind::EqEq, IrTy::I64) => Instruction::I64Eq,
        (BinOpKind::NotEq, IrTy::I64) => Instruction::I64Ne,

        (BinOpKind::Add, IrTy::F64) => Instruction::F64Add,
        (BinOpKind::Sub, IrTy::F64) => Instruction::F64Sub,
        (BinOpKind::Mul, IrTy::F64) => Instruction::F64Mul,
        (BinOpKind::Div, IrTy::F64) => Instruction::F64Div,
        (BinOpKind::Lt, IrTy::F64) => Instruction::F64Lt,
        (BinOpKind::Gt, IrTy::F64) => Instruction::F64Gt,
        (BinOpKind::LtEq, IrTy::F64) => Instruction::F64Le,
        (BinOpKind::GtEq, IrTy::F64) => Instruction::F64Ge,
        (BinOpKind::EqEq, IrTy::F64) => Instruction::F64Eq,
        (BinOpKind::NotEq, IrTy::F64) => Instruction::F64Ne,

        // bool operands; And/Or are eager here, not short circuiting
        (BinOpKind::And, IrTy::I32) => Instruction::I32And,
        (BinOpKind::Or, IrTy::I32) => Instruction::I32Or,
        (BinOpKind::EqEq, IrTy::I32) => Instruction::I32Eq,
        (BinOpKind::NotEq, IrTy::I32) => Instruction::I32Ne,

        (op, ty) => unreachable!("bug: sema let `{op}` on {ty:?} thru to codegen")
    }
}
