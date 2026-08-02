use crate::common::ast::BinOpKind;
use crate::sema::tast::StructId;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Unit,
    Array(Box<Ty>),
    Func { params: Vec<Ty>, ret: Box<Ty> },
    Struct(StructId),
    Infer
    // ...
}

impl Ty {
    fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float)
    }

    fn is_bool(&self) -> bool {
        matches!(self, Ty::Bool)
    }

    fn is_primitive(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float | Ty::Bool)
    }
}

pub fn bin_op_ty(op: BinOpKind, left: &Ty, right: &Ty) -> Option<Ty> {
    if left != right {
        return None;
    }

    use BinOpKind::*;
    match op {
        Add | Sub | Mul | Div if left.is_numeric() => Some(left.clone()),
        Lt | Gt | LtEq | GtEq if left.is_numeric() => Some(Ty::Bool),
        EqEq if left.is_primitive() => Some(Ty::Bool),
        And | Or | NotEq if left.is_bool() => Some(Ty::Bool),
        _ => None
    }
}