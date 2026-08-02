// prototype IR
// todo: decide IR design
#![allow(unused)]

use crate::common::op::{BinOpKind, UnaryOpKind};

pub type LocalId = u64;
pub type FuncId = u64;
pub type TypeId = u64;

#[derive(Debug)]
pub struct Module {
    pub heap_types: Vec<HeapType>,
    pub funcs: Vec<Func>,
    pub entry: Option<FuncId>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrTy {
    I32,
    I64,
    F64,
    Unit // erased by codegen (no wasm value)
}

#[derive(Debug)]
pub enum HeapType {
    Struct { fields: Vec<IrTy> }
}

#[derive(Debug)]
pub struct Func {
    pub id: FuncId,
    pub params: Vec<Param>,
    pub locals: Vec<IrTy>,
    pub ret_ty: IrTy,
    pub body: Expr
}

#[derive(Debug)]
pub struct Param {
    pub id: LocalId,
    pub ty: IrTy
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: IrTy
}

#[derive(Debug)]
pub enum ExprKind {
    ConstInt(i64),
    ConstFloat(f64),
    ConstBool(bool),
    Unit,
    LocalGet(LocalId),
    LocalSet { id: LocalId, value: Box<Expr> },
    Binary { op: BinOpKind, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOpKind, operand: Box<Expr> },
    Block(Vec<Expr>),
    Call(Call),
    FuncRef(FuncId),
    StructNew { ty: TypeId, fields: Vec<Expr> },
    StructGet {
        obj: Box<Expr>,
        ty: TypeId,
        field: u64
    }
}

#[derive(Debug)]
pub struct Call {
    pub func: FuncId,
    pub args: Vec<Expr>
}