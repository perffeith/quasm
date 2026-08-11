pub mod ir;

use crate::common::Literal;
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
            Ty::Void => IrTy::Void,
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
                tast::Stmt::ExternFunc(_) => todo!("emit wasm imports"),
                tast::Stmt::Func(func) => funcs.push(self.lower_func_decl(&func)),
                tast::Stmt::Struct(_) => todo!("implement struct ir"),
                _ => unreachable!("bug: sema let non top level stmt thru to lower")
            }
        }

        ir::Module {
            heap_types: vec![],
            funcs,
            entry: tast.entry.map(|id| ir::FuncId(id.0))
        }
    }

    fn lower_stmt(&mut self, stmt: &tast::Stmt) -> ir::Expr {
        match stmt {
            tast::Stmt::Local(local) => self.lower_local_decl(local.id, &local.value_ty, &local.value),
            tast::Stmt::Return(ret) => ir::Expr::Return(self.lower_return(ret)),
            tast::Stmt::If(if_stmt) => ir::Expr::If(self.lower_if(if_stmt)),
            tast::Stmt::Block(block) => ir::Expr::Block(self.lower_block(block)),
            tast::Stmt::Assign(assign) => {
                let value = self.lower_expr(&assign.value);
                ir::Expr::LocalSet(ir::LocalSet { id: ir::LocalId(assign.id.0), value: Box::new(value) })
            }
            tast::Stmt::Expr(expr) => self.lower_expr(expr),
            tast::Stmt::ExternFunc(_) => unreachable!("nested extern"),
            tast::Stmt::Func(_) => unreachable!("nested functions"),
            tast::Stmt::Struct(_) => unreachable!("nested structs")
        }
    }

    fn lower_block(&mut self, block: &tast::Block) -> ir::Block {
        let exprs = block.stmts.iter().map(|stmt| self.lower_stmt(stmt)).collect();
        ir::Block { exprs, ty: self.lower_ty(&block.ty) }
    }

    fn lower_func_decl(&mut self, func: &tast::Func) -> ir::Func {
        self.locals.clear();

        let params = func.params.iter()
            .map(|param| ir::Param { id: ir::LocalId(param.id.0), ty: self.lower_ty(&param.ty) })
            .collect();

        let body = self.lower_block(&func.body);

        ir::Func {
            id: ir::FuncId(func.id.0),
            params,
            locals: std::mem::take(&mut self.locals),
            ret_ty: self.lower_ty(&func.ret_ty),
            body
        }
    }

    fn lower_local_decl(&mut self, id: tast::VarId, value_ty: &Ty, value: &tast::Expr) -> ir::Expr {
        self.locals.push(self.lower_ty(value_ty));

        let value = self.lower_expr(value);
        ir::Expr::LocalSet(ir::LocalSet { id: ir::LocalId(id.0), value: Box::new(value) })
    }

    fn lower_return(&mut self, ret: &tast::Return) -> ir::Return {
        let value = ret.value.as_ref().map(|v| Box::new(self.lower_expr(v)));
        ir::Return { value: value }
    }

    fn lower_if(&mut self, if_stmt: &tast::If) -> ir::If {
        let ty = self.lower_ty(&if_stmt.ty);

        // Desugar the elif chain into nested ifs, built from the back
        let mut else_block: Option<Box<ir::Expr>> = if_stmt.else_block.as_ref()
            .map(|block| Box::new(ir::Expr::Block(self.lower_block(block))));

        for elif in if_stmt.elifs.iter().rev() {
            let condition = Box::new(self.lower_expr(&elif.condition));
            let then_block = Box::new(self.lower_block(&elif.then_block));
            else_block = Some(Box::new(ir::Expr::If(ir::If { condition, then_block, else_block, ty })));
        }

        let condition = Box::new(self.lower_expr(&if_stmt.condition));
        let then_block = Box::new(self.lower_block(&if_stmt.then_block));
        ir::If { condition, then_block, else_block, ty }
    }

    fn lower_expr(&mut self, expr: &tast::Expr) -> ir::Expr {
        match expr {
            tast::Expr::Literal(lit, _) => match lit {
                Literal::Int(v) => ir::Expr::ConstInt(*v),
                Literal::Float(v) => ir::Expr::ConstFloat(*v),
                Literal::Bool(v) => ir::Expr::ConstBool(*v)
            },
            tast::Expr::VarRef(var_ref) => {
                ir::Expr::LocalGet(ir::LocalGet { id: ir::LocalId(var_ref.id.0), ty: self.lower_ty(&var_ref.ty) })
            }
            tast::Expr::FuncRef(func_ref) => ir::Expr::FuncRef(ir::FuncId(func_ref.id.0)),
            tast::Expr::BinaryOp(binop) => {
                let ty = self.lower_ty(&binop.ty);
                let left = self.lower_expr(&binop.left);
                let right = self.lower_expr(&binop.right);
                ir::Expr::Binary(ir::Binary { op: binop.op, left: Box::new(left), right: Box::new(right), ty })
            }
            tast::Expr::UnaryOp(_) => todo!("unary ir"),
            tast::Expr::Call(call) => {
                let ty = self.lower_ty(&call.ty);
                let args = call.args.iter().map(|arg| self.lower_expr(arg)).collect();
                match call.callee.as_ref() {
                    tast::Expr::FuncRef(func_ref) => ir::Expr::Call(ir::Call { func: ir::FuncId(func_ref.id.0), args, ty }),
                    _ => todo!("other kind of calls?? bug or todo not sure tbh; gotta figure it out later")
                }
            }
            tast::Expr::StructLit(_) => todo!("struct literals")
        }
    }
}
