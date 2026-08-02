#![allow(unused)]
use std::collections::HashMap;
use wasm_encoder::{
    TypeSection,
    ValType
};
use crate::lower::ir;

pub struct Codegen {
    types: TypeSection,
    func_sig_cache: HashMap<(Vec<ValType>, Vec<ValType>), u32>
}

pub fn emit(module: ir::Module) -> Vec<u8> {
    Codegen::new();
    
    // placeholder
    vec![0]
}

impl Codegen {
    fn new() -> Self {
        Self {
            types: todo!(),
            func_sig_cache: HashMap::new()
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
}