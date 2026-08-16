// prototype IR
// todo: decide IR design
#![allow(unused)]

use crate::common::op::{BinOpKind, UnaryOpKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

#[derive(Debug)]
pub struct Module {
    pub heap_types: Vec<HeapType>,
    pub imports: Vec<ImportFunc>,
    pub funcs: Vec<Func>,
    pub entry: Option<FuncId>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrTy {
    I32,
    F32,
    Void // erased by codegen (no wasm value)
}

#[derive(Debug)]
pub enum HeapType {
    Struct { fields: Vec<IrTy> }
}

#[derive(Debug)]
pub struct ImportFunc {
    pub id: FuncId,
    pub module: String,
    pub item: String,
    pub params: Vec<IrTy>,
    pub ret_ty: IrTy
}

#[derive(Debug)]
pub struct Func {
    pub id: FuncId,
    pub params: Vec<Param>,
    pub locals: Vec<IrTy>,
    pub ret_ty: IrTy,
    pub body: Block
}

#[derive(Debug)]
pub struct Param {
    pub id: LocalId,
    pub ty: IrTy
}

#[derive(Debug)]
pub enum Expr {
    ConstInt(i32),
    ConstFloat(f32),
    ConstBool(bool),
    Void,
    LocalSet(LocalSet),
    LocalGet(LocalGet),
    Return(Return),
    Block(Block),
    If(If),
    Call(Call),
    FuncRef(FuncId),
    StructNew(StructNew),
    StructGet(StructGet),
    Binary(Binary),
    Unary(Unary)
}

impl Expr {
    pub fn ty(&self) -> IrTy {
        match self {
            Expr::ConstInt(_) => IrTy::I32,
            Expr::ConstFloat(_) => IrTy::F32,
            Expr::ConstBool(_) => IrTy::I32,
            Expr::Void | Expr::LocalSet(_) | Expr::Return(_) => IrTy::Void,
            Expr::LocalGet(e) => e.ty,
            Expr::Binary(e) => e.ty,
            Expr::Unary(e) => e.ty,
            Expr::Block(e) => e.ty,
            Expr::If(e) => e.ty,
            Expr::Call(e) => e.ty,
            // ref/heap types aren't modeled by IrTy yet
            Expr::FuncRef(_) | Expr::StructNew(_) | Expr::StructGet(_) =>
                todo!("no IrTy for ref/heap types yet")
        }
    }
}

#[derive(Debug)]
pub struct LocalSet {
    pub id: LocalId,
    pub value: Box<Expr>
}

#[derive(Debug)]
pub struct LocalGet {
    pub id: LocalId,
    pub ty: IrTy
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Box<Expr>>
}

#[derive(Debug)]
pub struct Block {
    pub exprs: Vec<Expr>,
    pub ty: IrTy
}

#[derive(Debug)]
pub struct If {
    pub condition: Box<Expr>,
    pub then_block: Box<Block>,
    // else is a Block, or a nested If for an elif branch
    pub else_block: Option<Box<Expr>>,
    pub ty: IrTy
}

#[derive(Debug)]
pub struct Call {
    pub func: FuncId,
    pub args: Vec<Expr>,
    pub ty: IrTy
}

#[derive(Debug)]
pub struct StructNew {
    pub ty: TypeId,
    pub fields: Vec<Expr>
}

#[derive(Debug)]
pub struct StructGet {
    pub obj: Box<Expr>,
    pub ty: TypeId,
    pub field: u64
}

#[derive(Debug)]
pub struct Binary {
    pub op: BinOpKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub ty: IrTy
}

#[derive(Debug)]
pub struct Unary {
    pub op: UnaryOpKind,
    pub operand: Box<Expr>,
    pub ty: IrTy
}