pub mod op;

use std::collections::HashMap;
use wasm_encoder::{
    CodeSection,
    EntityType,
    ExportKind,
    ExportSection,
    Function,
    FunctionSection,
    ImportSection,
    Module,
    TypeSection,
    ValType,
    Instruction
};
use crate::lower::ir;
use crate::lower::ir::IrTy;
use op::bin_op;

pub struct Codegen {
    types: TypeSection,
    imports: ImportSection,
    funcs: FunctionSection,
    exports: ExportSection,
    code: CodeSection,
    func_sig_cache: HashMap<(Vec<ValType>, Vec<ValType>), u32>
}

pub fn emit(module: ir::Module) -> Vec<u8> {
    Codegen::new().emit_module(module)
}

impl Codegen {
    fn new() -> Self {
        Self {
            types: TypeSection::new(),
            imports: ImportSection::new(),
            funcs: FunctionSection::new(),
            exports: ExportSection::new(),
            code: CodeSection::new(),
            func_sig_cache: HashMap::new()
        }
    }

    fn val_ty(&self, ty: IrTy) -> ValType {
        match ty {
            IrTy::I32 => ValType::I32,
            IrTy::F32 => ValType::F32,
            IrTy::Void => unreachable!("bug: Void has no wasm value type")
        }
    }

    fn param_tys(&self, tys: impl IntoIterator<Item = IrTy>) -> Vec<ValType> {
        tys.into_iter().map(|ty| self.val_ty(ty)).collect()
    }

    fn result_tys(&self, ty: IrTy) -> Vec<ValType> {
        match ty {
            IrTy::Void => vec![],
            ty => vec![self.val_ty(ty)]
        }
    }

    fn func_sig_index(&mut self, params: Vec<ValType>, results: Vec<ValType>) -> u32 {
        let key = (params.clone(), results.clone());
        *self.func_sig_cache.entry(key).or_insert_with(|| {
            let idx = self.types.len();
            self.types.ty().function(params, results);
            idx
        })
    }

    fn emit_module(mut self, module: ir::Module) -> Vec<u8> {
        // pass 1: declare imports
        for import in &module.imports {
            self.declare_import(import);
        }

        // pass 2: cache func sig
        for func in &module.funcs {
            self.declare_func(func);
        }

        if let Some(entry) = module.entry {
            self.exports.export("main", ExportKind::Func, entry.0);
        }

        // pass 3: emit func bodies
        for func in module.funcs {
            self.emit_func(func);
        }

        let mut wasm = Module::new();
        wasm.section(&self.types);
        wasm.section(&self.imports);
        wasm.section(&self.funcs);
        wasm.section(&self.exports);
        wasm.section(&self.code);
        wasm.finish()
    }

    fn declare_import(&mut self, import: &ir::ImportFunc) {
        let params = self.param_tys(import.params.iter().copied());
        let results = self.result_tys(import.ret_ty);
        let index = self.func_sig_index(params, results);
        self.imports.import(&import.module, &import.item, EntityType::Function(index));
    }

    fn declare_func(&mut self, func: &ir::Func) {
        let params = self.param_tys(func.params.iter().map(|param| param.ty));
        let results = self.result_tys(func.ret_ty);
        let index = self.func_sig_index(params, results);
        self.funcs.function(index);
    }

    fn emit_func(&mut self, func: ir::Func) {
        let locals: Vec<ValType> = func.locals.iter().map(|ty| self.val_ty(*ty)).collect();
        let mut func_body = Function::new_with_locals_types(locals);

        for expr in func.body.exprs {
            self.emit_stmt(&mut func_body, expr);
        }

        // if func.ret_ty != IrTy::Void {
        //     func_body.instruction(&Instruction::Unreachable);
        // }
        func_body.instruction(&Instruction::End);
        self.code.function(&func_body);
    }

    fn emit_stmt(&mut self, body: &mut Function, expr: ir::Expr) {
        let ty = expr.ty();
        self.emit_expr(body, expr);
        if ty != IrTy::Void {
            body.instruction(&Instruction::Drop);
        }
    }

    fn emit_expr(&mut self, body: &mut Function, expr: ir::Expr) {
        match expr {
            ir::Expr::ConstInt(value) => {
                body.instruction(&Instruction::I32Const(value));
            }
            ir::Expr::ConstBool(value) => {
                body.instruction(&Instruction::I32Const(value as i32));
            }
            ir::Expr::LocalGet(get) => {
                body.instruction(&Instruction::LocalGet(get.id.0));
            }
            ir::Expr::LocalSet(set) => {
                self.emit_expr(body, *set.value);
                body.instruction(&Instruction::LocalSet(set.id.0));
            }
            ir::Expr::Return(ret) => {
                if let Some(value) = ret.value {
                    self.emit_expr(body, *value);
                }
                body.instruction(&Instruction::Return);
            }
            ir::Expr::Call(call) => {
                for arg in call.args {
                    self.emit_expr(body, arg);
                }
                body.instruction(&Instruction::Call(call.func.0));
            }
            ir::Expr::Binary(binary) => {
                let operand_ty = binary.left.ty();
                self.emit_expr(body, *binary.left);
                self.emit_expr(body, *binary.right);
                body.instruction(&bin_op(binary.op, operand_ty));
            }
            e => todo!("codegen for {:?}", e)
        }
    }
}
