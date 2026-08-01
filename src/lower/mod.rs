pub mod ir;

use crate::common::ast::Literal;
use crate::sema::tast;
use crate::sema::ty::Ty;
use ir::{IrTy};

pub struct Lower {
    locals: Vec<IrTy>
}

pub fn lower(tast: tast::Program) -> ir::Module {
    Lower::new().lower_module(tast)
}

impl Lower {
    fn new() -> Self {
        Self {
            locals: Vec::new()
        }
    }

    fn lower_ty(&self, ty: &Ty) -> IrTy {
        match ty {
            Ty::Int => IrTy::I64,
            Ty::Float => IrTy::F64,
            Ty::Bool => IrTy::I32,
            Ty::Nil => IrTy::Unit,
            Ty::Struct(_) => todo!("struct types"),
            Ty::Array(_) => todo!("array types"),
            Ty::Func { .. } => todo!("function types"),
            Ty::Infer => unreachable!("bug: Ty::Infer survived sema")
        }
    }

    pub fn lower_module(&mut self, tast: tast::Program) -> ir::Module {
        let mut funcs = Vec::new();

        for stmt in tast.stmts {
            match stmt {
                tast::Stmt::Func(func) => funcs.push(self.lower_func_decl(&func)),
                tast::Stmt::Struct(_) => todo!("implement struct ir"),
                _ => unreachable!("bug: sema let non top level stmt thru to lower")
            }
        }

        ir::Module {
            heap_types: vec![],
            funcs,
            entry: tast.entry
        }
    }

    fn lower_block(&mut self, block: &tast::Block) -> ir::Expr {
        let exprs = block.stmts.iter().map(|stmt| self.lower_stmt(stmt)).collect();
        ir::Expr { kind: ir::ExprKind::Block(exprs), ty: self.lower_ty(&block.ty) }
    }

    fn lower_func_decl(&mut self, func: &tast::Func) -> ir::Func {
        self.locals.clear();

        let params = func.params.iter()
            .map(|param| ir::Param { id: param.id, ty: self.lower_ty(&param.ty) })
            .collect();

        let body = self.lower_block(&func.body);

        ir::Func {
            id: func.id,
            params,
            locals: std::mem::take(&mut self.locals),
            ret_ty: self.lower_ty(&func.ret_ty),
            body
        }
    }

    fn lower_stmt(&mut self, stmt: &tast::Stmt) -> ir::Expr {
        match stmt {
            tast::Stmt::Let(let_stmt) => self.lower_local_decl(let_stmt.id, &let_stmt.value_ty, &let_stmt.value),
            tast::Stmt::Expr(expr) => self.lower_expr(expr),
            tast::Stmt::Func(_) => unreachable!("nested functions"),
            tast::Stmt::Struct(_) => unreachable!("nested structs")
        }
    }

    fn lower_local_decl(&mut self, id: tast::VarId, value_ty: &Ty, value: &tast::Expr) -> ir::Expr {
        self.locals.push(self.lower_ty(value_ty));

        let value = self.lower_expr(value);
        ir::Expr {
            kind: ir::ExprKind::LocalSet { id, value: Box::new(value) },
            ty: IrTy::Unit
        }
    }

    fn lower_expr(&mut self, expr: &tast::Expr) -> ir::Expr {
        let ty = self.lower_ty(&expr.ty);

        let kind = match &expr.kind {
            tast::ExprKind::Literal(lit) => match lit {
                Literal::Int(v) => ir::ExprKind::ConstInt(*v),
                Literal::Float(v) => ir::ExprKind::ConstFloat(*v),
                Literal::Bool(v) => ir::ExprKind::ConstBool(*v),
                Literal::Nil => ir::ExprKind::Unit
            },
            tast::ExprKind::VarRef(id) => ir::ExprKind::LocalGet(*id),
            tast::ExprKind::FuncRef(id) => ir::ExprKind::FuncRef(*id),
            tast::ExprKind::Block(block) => return self.lower_block(block),
            tast::ExprKind::BinaryOp(binop) => {
                let left = self.lower_expr(&binop.left);
                let right = self.lower_expr(&binop.right);
                ir::ExprKind::Binary { op: binop.op, left: Box::new(left), right: Box::new(right) }
            }
            tast::ExprKind::UnaryOp(_) => todo!("unary ir"),
            tast::ExprKind::Call(call) => {
                let args = call.args.iter().map(|arg| self.lower_expr(arg)).collect();
                match &call.callee.kind {
                    tast::ExprKind::FuncRef(id) => ir::ExprKind::Call(ir::Call { func: *id, args }),
                    _ => todo!("other kind of calls?? bug or todo not sure tbh; gotta figure it out later")
                }
            }
            tast::ExprKind::StructLit(_) => todo!("struct literals"),
            tast::ExprKind::Error => unreachable!("bug: Error expr reached lower")
        };

        ir::Expr { kind, ty }
    }
}
