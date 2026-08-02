use crate::common::ast::{Literal, BinOpKind, UnaryOpKind};
use crate::sema::ty::Ty;
use crate::common::span::Span;

pub type VarId = u64;
pub type FuncId = u64;
pub type StructId = u64;
pub type StructFieldId = u64;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub entry: Option<FuncId>
}

#[derive(Debug)]
pub enum Stmt {
    Func(Func),
    Struct(Struct),
    Let(Let),
    Return(Return),
    Assign(Assign),
    Expr(Expr)
}

#[derive(Debug)]
pub struct Func {
    pub id: FuncId,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
    pub body: Block
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Expr>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Param {
    pub id: VarId,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Struct {
    pub id: StructId,
    pub fields: Vec<StructField>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct StructField {
    pub id: StructFieldId,
    pub ty: Ty
}

#[derive(Debug)]
pub struct StructLit {
    pub id: StructId,
    pub fields: Vec<Expr>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Let {
    pub id: VarId,
    pub is_mut: bool,
    pub value: Expr,
    pub value_ty: Ty,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Assign {
    pub id: VarId,
    pub value: Expr,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Ty
}

#[derive(Debug)]
pub enum ExprKind {
    Literal(Literal),
    VarRef(VarId),
    FuncRef(FuncId),
    Block(Block),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    Call(Call),
    StructLit(StructLit),
    // stands in for an expression that failed to check or isn't supported yet
    Error
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub ty: Ty,
    pub span: Span
}

#[derive(Debug)]
pub struct BinaryOp {
    pub op: BinOpKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct UnaryOp {
    pub op: UnaryOpKind,
    pub operand: Box<Expr>
}

#[derive(Debug)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub ty: Ty
}