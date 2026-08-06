use crate::common::Literal;
use crate::common::op::{BinOpKind, UnaryOpKind};
use crate::sema::ty::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructFieldId(pub u32);

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub entry: Option<FuncId>
}

#[derive(Debug)]
pub enum Stmt {
    Func(Func),
    Struct(Struct),
    Local(Local),
    Return(Return),
    If(If),
    Assign(Assign),
    Block(Block),
    Expr(Expr)
}

impl Stmt {
    // whether control flow always leaves this statement via a return
    pub fn always_returns(&self) -> bool {
        match self {
            Stmt::Return(_) => true,
            Stmt::Block(block) => block.always_returns(),
            Stmt::If(if_stmt) => {
                let Some(else_block) = &if_stmt.else_block else {
                    return false;
                };
                if_stmt.then_block.always_returns()
                    && if_stmt.elifs.iter().all(|elif| elif.then_block.always_returns())
                    && else_block.always_returns()
            }
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct Func {
    pub id: FuncId,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
    pub body: Block
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
pub struct Local {
    pub id: VarId,
    pub value: Expr,
    pub value_ty: Ty,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Expr>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct If {
    pub condition: Expr,
    pub then_block: Block,
    pub elifs: Vec<Elif>,
    pub else_block: Option<Block>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Elif {
    pub condition: Expr,
    pub then_block: Block
}

#[derive(Debug)]
pub struct Assign {
    pub id: VarId,
    pub value: Expr,
    pub ty: Ty
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal, Ty),
    FuncRef(FuncRef),
    VarRef(VarRef),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    Call(Call),
    StructLit(StructLit)
}

impl Expr {
    pub fn ty(&self) -> &Ty {
        match self {
            Expr::Literal(_, ty) => ty,
            Expr::FuncRef(e) => &e.ty,
            Expr::VarRef(e) => &e.ty,
            Expr::BinaryOp(e) => &e.ty,
            Expr::UnaryOp(e) => &e.ty,
            Expr::Call(e) => &e.ty,
            Expr::StructLit(e) => &e.ty
        }
    }
}

#[derive(Debug)]
pub struct FuncRef {
    pub id: FuncId,
    pub ty: Ty
}

#[derive(Debug)]
pub struct VarRef {
    pub id: VarId,
    pub ty: Ty
}


#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub ty: Ty
}

impl Block {
    // whether control flow always leaves this block via a return
    pub fn always_returns(&self) -> bool {
        self.stmts.iter().any(Stmt::always_returns)
    }
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
    pub operand: Box<Expr>,
    pub ty: Ty
}

#[derive(Debug)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub ty: Ty
}
