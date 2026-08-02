use crate::common::span::Span;
use crate::common::ast::{Literal, BinOpKind, UnaryOpKind};

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>
}

#[derive(Debug)]
pub enum Stmt {
    Func(Func),
    Struct(Struct),
    Type(Type),
    Let(Let),
    Return(Return),
    If(If),
    Assign(Assign),
    Expr(Expr)
}

#[derive(Debug)]
pub struct Func {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    pub body: Block
}

#[derive(Debug)]
pub struct Param {
    pub name: Identifier,
    pub ty: Option<Ty>
}

#[derive(Debug)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<StructField>
}

#[derive(Debug)]
pub struct StructField {
    pub name: Identifier,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Type {
    pub name: Identifier,
    pub variants: Vec<TypeVariant>
}

#[derive(Debug)]
pub struct TypeVariant {
    pub name: Identifier,
    pub ty_fields: Vec<Ty>
}

#[derive(Debug)]
pub struct Let {
    pub name: Identifier,
    pub is_mut: bool,
    pub value: Expr,
    pub annot_ty: Option<Ty>
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Expr>,
    pub span: Span
}

#[derive(Debug)]
pub struct If {
    pub condition: Expr,
    pub then_block: Block,
    pub else_branch: Option<Else>
}

#[derive(Debug)]
pub struct Assign {
    pub target: Expr,
    pub value: Expr
}

#[derive(Debug)]
pub enum Else {
    ElseIf(Box<If>),
    Block(Block)
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span
}

#[derive(Debug)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(Identifier),
    Array(Vec<Expr>),
    Block(Block),
    BinaryOp(BinaryOp),
    UnaryOp {
        op: UnaryOpKind,
        operand: Box<Expr>
    },
    Call(Call),
    // `base.name(args)`. sema decides UFCS call or variant constructor
    // Never calls a stored field. But if we want to it should be: `(base.name)(args)`
    DotCall {
        base: Box<Expr>,
        callee: Identifier,
        args: Vec<Expr>
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>
    },
    // `base.name` sema decides: struct field or variant reference
    DotAccess {
        base: Box<Expr>,
        name: Identifier
    },
    Closure {
        params: Vec<Param>,
        ret: Option<Ty>,
        body: Box<Expr>
    }
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span
}

#[derive(Debug)]
pub struct BinaryOp {
    pub op: BinOpKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>
}

#[derive(Debug)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>
}

#[derive(Debug)]
pub struct Identifier {
    pub value: String,
    pub span: Span
}

#[derive(Debug)]
pub enum Ty {
    Named {
        name: Identifier
    },
    Array(Box<Ty>),
    Func {
        params: Vec<Ty>,
        ret: Option<Box<Ty>>
    }
}