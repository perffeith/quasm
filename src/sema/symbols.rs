use std::collections::HashMap;
use indexmap::IndexMap;
use crate::sema::{ty::Ty};
use crate::sema::tast::{FuncId, StructId, VarId};

// ----helpers----
fn is_snake_case(name: &str) -> bool {
    let starts_ok = name.chars().next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_');
    starts_ok && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_pascal_case(name: &str) -> bool {
    let starts_ok = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
    starts_ok && name.chars().all(|c| c.is_ascii_alphanumeric())
}

#[derive(PartialEq, Eq, Hash)]
struct FuncKey {
    name: String,
    first_param_ty: Option<Ty> // None for func with no param
}

pub struct FuncSymbol {
    pub id: FuncId,
    pub params_ty: Vec<Ty>,
    pub ret_ty: Ty
}

pub struct StructSymbol {
    pub id: StructId,
    pub fields: IndexMap<String, Ty>
}

pub struct VarSymbol {
    pub id: VarId,
    pub ty: Ty
}

// ----Symbol Table----
pub struct SymbolTable {
    funcs: HashMap<FuncKey, FuncSymbol>,
    struct_ids: HashMap<String, StructId>,
    structs: HashMap<StructId, StructSymbol>,
    scopes: Vec<HashMap<String, VarSymbol>>,
    cur_local_id: u64,
    cur_ret_ty: Ty
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new(),
            struct_ids: HashMap::new(),
            structs: HashMap::new(),
            scopes: Vec::new(),
            cur_local_id: 0,
            cur_ret_ty: Ty::Void
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    // ----Function related stuffs----
    pub fn define_func(&mut self, name: &str, params_ty: Vec<Ty>, ret_ty: Ty) -> Result<(), String> {
        if !is_snake_case(name) {
            return Err(format!("function `{name}` must be snake_case"));
        }

        let key = FuncKey {
            name: name.to_string(),
            first_param_ty: params_ty.first().cloned()
        };

        if self.funcs.contains_key(&key) {
            let signature = match &key.first_param_ty {
                Some(ty) => format!("`{name}({ty:?}...)`"),
                None => format!("`{name}`"),
            };
            return Err(format!("function {signature} is already defined"));
        }

        let id = FuncId(self.funcs.len() as u64);

        self.funcs.insert(key, FuncSymbol { id, params_ty, ret_ty });
        Ok(())
    }

    pub fn lookup_func(&self, name: &str, first_param_ty: Option<Ty>) -> Option<&FuncSymbol> {
        self.funcs.get(&FuncKey { name: name.to_string(), first_param_ty })
    }

    pub fn enter_func(&mut self, ret_ty: Ty) {
        self.enter_scope();
        self.cur_local_id = 0;
        self.cur_ret_ty = ret_ty;
    }

    pub fn exit_func(&mut self) {
        self.exit_scope();
        self.cur_ret_ty = Ty::Void;
    }

    pub fn cur_ret_ty(&self) -> &Ty {
        &self.cur_ret_ty
    }

    fn alloc_local_id(&mut self) -> VarId {
        let id = VarId(self.cur_local_id);
        self.cur_local_id += 1;
        id
    }

    // ----Struct related stuffs----
    pub fn define_struct(&mut self, name: &str) -> Result<StructId, String> {
        if !is_pascal_case(name) {
            return Err(format!("struct `{name}` must be PascalCase"));
        }

        if self.struct_ids.contains_key(name) {
            return Err(format!("struct {name} is already defined"));
        }

        let id = StructId(self.struct_ids.len() as u64);
        self.struct_ids.insert(name.to_string(), id);
        Ok(id)
    }

    pub fn define_struct_fields(&mut self, name: &str, fields: &[(String, Ty)]) -> Result<StructId, String> {
        let id = *self.struct_ids.get(name)
            .expect("bug: getting struct id has failed, something wrong with pass 1");

        let mut map = IndexMap::new();
        for (fname, ty) in fields {
            if map.contains_key(fname) {
                return Err(format!("field `{fname}` is already defined"));
            }
            map.insert(fname.clone(), ty.clone());
        }

        self.structs.insert(id, StructSymbol { id, fields: map });
        Ok(id)
    }

    pub fn lookup_struct_id(&self, name: &str) -> Option<StructId> {
        self.struct_ids.get(name).copied()
    }

    pub fn lookup_struct(&self, name: &str) -> Option<&StructSymbol> {
        self.structs.get(&self.lookup_struct_id(name)?)
    }

    // ----Variable related stuffs----
    pub fn define_var(&mut self, name: &str, ty: Ty) -> Result<VarId, String> {
        if !is_snake_case(name) {
            return Err(format!("variable `{name}` must be snake_case"));
        }

        let id = self.alloc_local_id();

        let scope = self.scopes.last_mut()
            .expect("bug: idk why but for some reason define_var is called without any scope");

        if scope.contains_key(name) {
            return Err(format!("variable `{name}` is already defined"))
        }

        scope.insert(name.to_string(), VarSymbol { id, ty });
        Ok(id)
    }

    pub fn lookup_var(&self, name: &str) -> Option<&VarSymbol> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }
}
