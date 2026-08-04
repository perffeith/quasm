use std::collections::HashMap;
use wasm_encoder::{
    CodeSection,
    ExportKind,
    ExportSection,
    Function,
    FunctionSection,
    Module,
    TypeSection,
    ValType,
    Instruction
};
use crate::lower::ir;
use crate::lower::ir::IrTy;

pub struct Codegen {
    types: TypeSection,
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
            funcs: FunctionSection::new(),
            exports: ExportSection::new(),
            code: CodeSection::new(),
            func_sig_cache: HashMap::new()
        }
    }

    fn val_ty(&self, ty: IrTy) -> ValType {
        match ty {
            IrTy::I32 => ValType::I32,
            IrTy::I64 => ValType::I64,
            IrTy::F64 => ValType::F64,
            IrTy::Void => unreachable!("bug: Void has no wasm value type")
        }
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
        // pas 1: cache func sig
        for func in &module.funcs {
            self.declare_func(func);
        }

        if let Some(entry) = module.entry {
            self.exports.export("main", ExportKind::Func, entry.0 as u32);
        }

        // pass 2: emit func bodies
        for func in &module.funcs {
            self.emit_func(func);
        }

        let mut wasm = Module::new();
        wasm.section(&self.types);
        wasm.section(&self.funcs);
        wasm.section(&self.exports);
        wasm.section(&self.code);
        wasm.finish()
    }

    fn declare_func(&mut self, func: &ir::Func) {
        let params = func.params.iter().map(|param| self.val_ty(param.ty)).collect();
        let results = self.result_tys(func.ret_ty);
        let index = self.func_sig_index(params, results);

        self.funcs.function(index);
    }

    fn emit_func(&mut self, _func: &ir::Func) {
        // for expr in &func.body.exprs {

        // }
        // todo: lower func.body.exprs into instructions
        let mut body = Function::new([]);
        body.instruction(&Instruction::Unreachable);
        body.instruction(&Instruction::End);
        self.code.function(&body);
    }
}
