pub mod ty;
pub mod tast;
pub mod symbols;

use ty::Ty;
use crate::parser::ast;
use crate::common::ast::Literal;
use crate::common::span::Span;
use symbols::SymbolTable;

pub struct Sema {
    sym_table: SymbolTable
}

#[derive(Debug)]
pub struct SemaError {
    pub message: String,
    pub span: Span
}

pub fn check(ast: ast::Program) -> Result<tast::Program, SemaError> {
    Sema::new().check_program(ast)
}

impl Sema {
    fn new() -> Self {
        Self {
            sym_table: SymbolTable::new()
        }
    }

    fn err(&self, message: impl Into<String>, span: Span) -> SemaError {
        SemaError { message: message.into(), span }
    }

    fn expect_eq(&self, expected: &Ty, got: &Ty, span: Span,
        context: impl FnOnce() -> String,
    ) -> Result<(), SemaError> {
        if expected != got {
            return Err(self.err(
                format!("{}: expected `{:?}`, got `{:?}`", context(), expected, got),
                span,
            ));
        }
        Ok(())
    }

    fn resolve_ty(&self, ty: &ast::Ty) -> Result<Ty, SemaError> {
        match ty {
            ast::Ty::Named { name, args } => {
                if !args.is_empty() {
                    return Err(self.err("generic types are not supported yet", name.span));
                }

                match name.value.as_str() {
                    "Int" => Ok(Ty::Int),
                    "Float" => Ok(Ty::Float),
                    "Bool" => Ok(Ty::Bool),
                    "Nil" => Ok(Ty::Nil),
                    _ => Err(self.err(format!("unknown type `{}`", name.value), name.span))
                }
            }
            ast::Ty::Array(inner) => {
                Ok(Ty::Array(Box::new(self.resolve_ty(inner)?)))
            }
            ast::Ty::Func { params, ret } => {
                let params = params.iter().map(|p| self.resolve_ty(p)).collect::<Result<_,_>>()?;
                let ret = match ret {
                    Some(r) => self.resolve_ty(r)?,
                    None => Ty::Nil
                };
                Ok(Ty::Func { params, ret: Box::new(ret) })
            }
        }
    }

    fn resolve_func_params_ty(&self, func: &ast::Func) -> Result<Vec<Ty>, SemaError> {
        let mut params_ty = Vec::new();
        for param in &func.params {
            let param_ty = match &param.ty {
                Some(ty) => self.resolve_ty(ty)?,
                None => Ty::Infer
            };
            params_ty.push(param_ty);
        }
        Ok(params_ty)
    }

    fn check_program(&mut self, ast: ast::Program) -> Result<tast::Program, SemaError> {
        // pass 1: register struct names, reject invalid top level statements
        for stmt in &ast.stmts {
            match stmt {
                ast::Stmt::Func(_) => {}
                ast::Stmt::Struct(struc) => {
                    if !struc.ty_params.is_empty() {
                        return Err(self.err("generic structs are not supported yet", struc.name.span));
                    }
                    self.sym_table.define_struct(&struc.name.value)
                        .map_err(|msg| self.err(msg, struc.name.span))?;
                }
                ast::Stmt::Let(s) => {
                    return Err(self.err("top level should not contain let statement", s.name.span));
                }
                ast::Stmt::Assign(s) => {
                    return Err(self.err("top level should not contain assignment", s.target.span));
                }
                ast::Stmt::Type(s) => {
                    return Err(self.err("not implemented yet", s.name.span));
                }
                ast::Stmt::Expr(e) => {
                    return Err(self.err("top level should not contain expression", e.span));
                }
            }
        }

        // pass 2: resolve func signatures and struct fields
        for stmt in &ast.stmts {
            match stmt {
                ast::Stmt::Func(func) => {
                    let name = func.name.value.clone();
                    let params_ty = self.resolve_func_params_ty(func)?;
                    let ret = match &func.ret {
                        Some(ret) => self.resolve_ty(ret)?,
                        None => Ty::Nil
                    };
                    self.sym_table.define_func(&name, params_ty, ret)
                        .map_err(|msg| self.err(msg, func.name.span))?;
                }
                ast::Stmt::Struct(struc) => {
                    let mut fields = Vec::new();
                    for field in &struc.fields {
                        let ty = self.resolve_ty(&field.ty)?;
                        fields.push((field.name.value.clone(), ty));
                    }
                    self.sym_table.define_struct_fields(&struc.name.value, &fields)
                        .map_err(|msg| self.err(msg, struc.name.span))?;
                }
                _ => {}
            }
        }

        // pass 3: build tast
        let mut stmts = Vec::new();

        for stmt in ast.stmts {
            stmts.push(self.check_stmt(stmt)?);
        }

        let entry = self.sym_table.lookup_func("main", None).map(|func| func.id);

        Ok(tast::Program { stmts, entry })
    }

    fn check_stmt(&mut self, stmt: ast::Stmt) -> Result<tast::Stmt, SemaError> {
        match stmt {
            ast::Stmt::Func(func) => Ok(tast::Stmt::Func(self.check_func_decl(func)?)),
            ast::Stmt::Struct(struc) => Ok(tast::Stmt::Struct(self.check_struct_decl(struc)?)),
            ast::Stmt::Let(let_stmt) => Ok(tast::Stmt::Let(self.check_let(let_stmt)?)),
            ast::Stmt::Assign(assign) => Ok(tast::Stmt::Assign(self.check_assign(assign)?)),
            ast::Stmt::Type(type_stmt) => {
                Err(self.err("not implemented yet", type_stmt.name.span))
            }
            ast::Stmt::Expr(expr) => Ok(tast::Stmt::Expr(self.check_expr(expr)?))
        }
    }

    fn check_func_decl(&mut self, func: ast::Func) -> Result<tast::Func, SemaError> {
        // lookup symbol table
        let name = func.name.value;
        let first_param_ty = func.params.first()
            .map(|param| self.resolve_ty(param.ty.as_ref()
            .expect("bug: func decl param ended up without a type annot. WHAT?")))
            .transpose()?;

        let Some(func_symbol) = self.sym_table.lookup_func(&name, first_param_ty) else {
            return Err(self.err(format!("function `{}` is not declared", name), func.name.span));
        };
        let id = func_symbol.id;
        let params_ty = func_symbol.params_ty.clone();
        let ret_ty = func_symbol.ret_ty.clone();

        // enter func and build params
        self.sym_table.enter_func();
        let mut params = Vec::new();
        for (param, ty) in func.params.iter().zip(params_ty) {
            let id = self.sym_table.define_var(&param.name.value, false, ty.clone())
                .map_err(|msg| self.err(msg, param.name.span))?;
            params.push(tast::Param { id, ty });
        }

        // build body
        let body = self.check_block(func.body)?;
        self.sym_table.exit_func();

        self.expect_eq(&ret_ty, &body.ty, body.span, || {
            format!("type mismatch for function `{}` return type", name)
        })?;

        Ok(tast::Func { id, params, ret_ty, body })
    }

    fn check_struct_decl(&mut self, struc: ast::Struct) -> Result<tast::Struct, SemaError> {
        let symbol = self.sym_table.lookup_struct(&struc.name.value)
            .expect("bug: struct fields were not resolved in pass 1 and 2");
        let id = symbol.id;

        let fields = symbol.fields.values().enumerate()
            .map(|(i, ty)| tast::StructField {
                id: i as tast::StructFieldId,
                ty: ty.clone()
            }).collect();

        Ok(tast::Struct { id, fields, ty: Ty::Nil })
    }

    fn check_let(&mut self, let_stmt: ast::Let) -> Result<tast::Let, SemaError> {
        let value = self.check_expr(let_stmt.value)?;

        let value_ty = match &let_stmt.annot_ty {
            Some(annot) => {
                let annot_ty = self.resolve_ty(annot)?;
                self.expect_eq(&annot_ty, &value.ty, let_stmt.name.span, || {
                    format!("type mismatch for `{}`", let_stmt.name.value)
                })?;
                annot_ty
            }
            None => value.ty.clone()
        };

        let id = self.sym_table.define_var(&let_stmt.name.value, let_stmt.is_mut, value_ty.clone())
            .map_err(|msg| self.err(msg, let_stmt.name.span))?;

        Ok(tast::Let { id, is_mut: let_stmt.is_mut, value, value_ty, ty: Ty::Nil })
    }

    fn check_assign(&mut self, assign: ast::Assign) -> Result<tast::Assign, SemaError> {
        let ast::ExprKind::Identifier(target) = assign.target.kind else {
            return Err(self.err("invalid assignment target", assign.target.span));
        };

        let Some(var_symbol) = self.sym_table.lookup_var(&target.value) else {
            return Err(self.err(
                format!("cannot find `{}` in this scope", target.value),
                target.span
            ));
        };
        let id = var_symbol.id;
        let is_mut = var_symbol.is_mut;
        let target_ty = var_symbol.ty.clone();

        if !is_mut {
            return Err(self.err(
                format!("cannot assign to immutable variable `{}`", target.value),
                target.span
            ));
        }

        let value = self.check_expr(assign.value)?;
        self.expect_eq(&target_ty, &value.ty, target.span, || {
            format!("type mismatch for `{}`", target.value)
        })?;

        Ok(tast::Assign { id, value, ty: Ty::Nil })
    }

    fn check_expr(&mut self, expr: ast::Expr) -> Result<tast::Expr, SemaError> {
        match expr.kind {
            ast::ExprKind::Literal(lit) => {
                let ty = match lit {
                    Literal::Int(_) => Ty::Int,
                    Literal::Float(_) => Ty::Float,
                    Literal::Bool(_) => Ty::Bool,
                    Literal::Nil => Ty::Nil
                };
                Ok(tast::Expr { kind: tast::ExprKind::Literal(lit), ty })
            }
            ast::ExprKind::Block(block) => {
                let block = self.check_block(block)?;
                let ty = block.ty.clone();
                Ok(tast::Expr { kind: tast::ExprKind::Block(block), ty })
            }
            ast::ExprKind::Identifier(identifier) => {
                let Some(var_symbol) = self.sym_table.lookup_var(&identifier.value) else {
                    return Err(self.err(
                        format!("cannot find `{}` in this scope", identifier.value),
                        identifier.span
                    ));
                };
                Ok(tast::Expr { kind: tast::ExprKind::VarRef(var_symbol.id), ty: var_symbol.ty.clone() })
            }
            ast::ExprKind::BinaryOp(binaryop) => {
                let binaryop = self.check_binaryop(binaryop)?;
                let ty = binaryop.ty.clone();
                Ok(tast::Expr { kind: tast::ExprKind::BinaryOp(binaryop), ty })
            }
            ast::ExprKind::Call(call) => self.check_call(call),
            _ => Ok(tast::Expr { kind: tast::ExprKind::Error, ty: Ty::Nil })
        }
    }

    fn check_block(&mut self, block: ast::Block) -> Result<tast::Block, SemaError> {
        let span = block.span;

        self.sym_table.enter_scope();
        let mut stmts = Vec::new();
        for stmt in block.stmts {
            stmts.push(self.check_stmt(stmt)?);
        }
        self.sym_table.exit_scope();

        // a block evaluates to its trailing expression, otherwise to unit
        let ty = match stmts.last() {
            Some(tast::Stmt::Expr(expr)) => expr.ty.clone(),
            _ => Ty::Nil
        };

        Ok(tast::Block { stmts, ty, span })
    }

    fn check_binaryop(&mut self, binaryop: ast::BinaryOp) -> Result<tast::BinaryOp, SemaError> {
        let span = binaryop.left.span;
        let left = self.check_expr(*binaryop.left)?;
        let right = self.check_expr(*binaryop.right)?;

        let Some(ty) = ty::bin_op_ty(binaryop.op, &left.ty, &right.ty) else {
            return Err(self.err(
                format!("invalid binary operation: `{:?}` {} `{:?}`", left.ty, binaryop.op, right.ty),
                span
            ));
        };

        Ok(tast::BinaryOp { op: binaryop.op, left: Box::new(left), right: Box::new(right), ty })
    }

    fn check_call(&mut self, call: ast::Call) -> Result<tast::Expr, SemaError> {
        let span = call.callee.span;

        let mut args = Vec::new();
        for arg in call.args {
            args.push(self.check_expr(arg)?);
        }

        match call.callee.kind {
            ast::ExprKind::Identifier(identifier) => {
                let name = identifier.value;

                // PascalCase names resolve to struct literal
                if self.sym_table.lookup_struct(&name).is_some() {
                    return self.check_struct_lit(&name, args, span);
                }

                // otherwise it's a function call
                let first_param_ty = args.first().map(|arg| arg.ty.clone());

                let Some(func_symbol) = self.sym_table.lookup_func(&name, first_param_ty) else {
                    return Err(self.err(
                        format!("cannot find function `{}`", name),
                        identifier.span
                    ));
                };
                let id = func_symbol.id;
                let params_ty = func_symbol.params_ty.clone();
                let ret_ty = func_symbol.ret_ty.clone();

                // validate params
                if args.len() != params_ty.len() {
                    return Err(self.err(
                        format!(
                            "function `{}` expects {} argument(s), got {}",
                            name, params_ty.len(), args.len()
                        ),
                        span
                    ));
                }

                for (arg, param_ty) in args.iter().zip(&params_ty) {
                    self.expect_eq(param_ty, &arg.ty, span, || {
                        format!("type mismatch in call to `{}`", name)
                    })?;
                }

                // build tast
                let callee = tast::Expr {
                    kind: tast::ExprKind::FuncRef(id) ,
                    ty: Ty::Func { params: params_ty, ret: Box::new(ret_ty.clone()) }
                };

                Ok(tast::Expr {
                    kind: tast::ExprKind::Call(tast::Call {
                        callee: Box::new(callee), args, ty: ret_ty.clone()
                    }),
                    ty: ret_ty
                })
            },
            _ => {
                Err(self.err("only call on identifier is supported", span))
            }
        }
    }

    fn check_struct_lit(&self, name: &str, args: Vec<tast::Expr>, span: Span) -> Result<tast::Expr, SemaError> {
        let struct_symbol = self.sym_table.lookup_struct(name)
            .expect("bug: struct doesn't exist on sym table");
        let id = struct_symbol.id;
        let field_tys: Vec<Ty> = struct_symbol.fields.values().cloned().collect();

        if args.len() != field_tys.len() {
            return Err(self.err(
                format!(
                    "struct `{}` expects {} field(s), got {}",
                    name, field_tys.len(), args.len()
                ),
                span
            ));
        }

        for (arg, field_ty) in args.iter().zip(&field_tys) {
            self.expect_eq(field_ty, &arg.ty, span, || {
                format!("type mismatch in struct literal `{}`", name)
            })?;
        }

        Ok(tast::Expr {
            kind: tast::ExprKind::StructLit(tast::StructLit { id, fields: args, ty: Ty::Struct(id) }),
            ty: Ty::Struct(id)
        })
    }
}