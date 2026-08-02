use crate::common::span::Span;
use crate::common::Literal;
use crate::common::op::{BinOpKind, UnaryOpKind};

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>
}

#[derive(Debug)]
pub enum Stmt {
    Func(Func),
    Struct(Struct),
    Enum(Enum),
    Let(Let),
    Return(Return),
    If(If),
    Assign(Assign),
    Expr(Expr)
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Func(s) => s.span,
            Stmt::Struct(s) => s.span,
            Stmt::Enum(s) => s.span,
            Stmt::Let(s) => s.span,
            Stmt::Return(s) => s.span,
            Stmt::If(s) => s.span,
            Stmt::Assign(s) => s.span,
            Stmt::Expr(e) => e.span()
        }
    }
}

#[derive(Debug)]
pub struct Func {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    pub body: Block,
    pub span: Span
}

#[derive(Debug)]
pub struct Param {
    pub name: Identifier,
    pub ty: Option<Ty>,
    pub span: Span
}

#[derive(Debug)]
pub struct Struct {
    pub name: Identifier,
    pub fields: Vec<StructField>,
    pub span: Span
}

#[derive(Debug)]
pub struct StructField {
    pub name: Identifier,
    pub ty: Ty,
    pub span: Span
}

#[derive(Debug)]
pub struct Enum {
    pub name: Identifier,
    pub variants: Vec<EnumVariant>,
    pub span: Span
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: Identifier,
    pub ty_fields: Vec<Ty>,
    pub span: Span
}

#[derive(Debug)]
pub struct Let {
    pub name: Identifier,
    pub is_mut: bool,
    pub value: Expr,
    pub annot_ty: Option<Ty>,
    pub span: Span
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
    pub elifs: Vec<Elif>,
    pub else_block: Option<Block>,
    pub span: Span
}

#[derive(Debug)]
pub struct Elif {
    pub condition: Expr,
    pub then_block: Block,
    pub span: Span
}

#[derive(Debug)]
pub struct Assign {
    pub target: Expr,
    pub value: Expr,
    pub span: Span
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal, Span),
    Identifier(Identifier),
    Array(Vec<Expr>, Span),
    Block(Block),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    Call(Call),
    // `base.name(args)`. sema decides UFCS call or variant constructor
    // Never calls a stored field. But if we want to it should be: `(base.name)(args)`
    DotCall(DotCall),
    Index(Index),
    // `base.name` sema decides: struct field or variant reference
    DotAccess(DotAccess),
    Closure(Closure)
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Identifier(ident) => ident.span,
            Expr::Array(_, span) => *span,
            Expr::Block(block) => block.span,
            Expr::BinaryOp(e) => e.span,
            Expr::UnaryOp(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::DotCall(e) => e.span,
            Expr::Index(e) => e.span,
            Expr::DotAccess(e) => e.span,
            Expr::Closure(e) => e.span
        }
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
    pub right: Box<Expr>,
    pub span: Span
}

#[derive(Debug)]
pub struct UnaryOp {
    pub op: UnaryOpKind,
    pub operand: Box<Expr>,
    pub span: Span
}

#[derive(Debug)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span
}

#[derive(Debug)]
pub struct DotCall {
    pub base: Box<Expr>,
    pub callee: Identifier,
    pub args: Vec<Expr>,
    pub span: Span
}

#[derive(Debug)]
pub struct Index {
    pub base: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span
}

#[derive(Debug)]
pub struct DotAccess {
    pub base: Box<Expr>,
    pub name: Identifier,
    pub span: Span
}

#[derive(Debug)]
pub struct Closure {
    pub params: Vec<Param>,
    pub ret: Option<Ty>,
    pub body: Box<Expr>,
    pub span: Span
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