use std::collections::HashMap;
use indexmap::IndexMap;
use crate::common::span::Span;
use crate::parser::ast;
use crate::sema::SemaError;
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
    funcs: HashMap<String, FuncSymbol>,
    struct_ids: HashMap<String, StructId>,
    structs: HashMap<StructId, StructSymbol>,
    scopes: Vec<HashMap<String, VarSymbol>>,
    cur_local_id: u32,
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

    fn err(&self, message: impl Into<String>, span: Span) -> SemaError {
        SemaError { message: message.into(), span }
    }

    // ----Type resolution----
    pub fn resolve_ty(&self, ty: &ast::Ty) -> Result<Ty, SemaError> {
        match ty {
            ast::Ty::Named { name } => {
                match name.value.as_str() {
                    "Int" => Ok(Ty::Int),
                    "Float" => Ok(Ty::Float),
                    "Bool" => Ok(Ty::Bool),
                    _ => match self.lookup_struct_id(&name.value) {
                        Some(id) => Ok(Ty::Struct(id)),
                        None => Err(self.err(format!("unknown type `{}`", name.value), name.span))
                    }
                }
            }
            ast::Ty::Array(inner) => {
                Ok(Ty::Array(Box::new(self.resolve_ty(inner)?)))
            }
            ast::Ty::Func { params, ret } => {
                let params = params.iter().map(|p| self.resolve_ty(p)).collect::<Result<_,_>>()?;
                let ret = match ret {
                    Some(r) => self.resolve_ty(r)?,
                    None => Ty::Void
                };
                Ok(Ty::Func { params, ret: Box::new(ret) })
            }
        }
    }

    fn resolve_params_ty(&self, params: &[ast::Param]) -> Result<Vec<Ty>, SemaError> {
        params.iter().map(|param| match &param.ty {
            Some(ty) => self.resolve_ty(ty),
            None => Ok(Ty::Infer)
        }).collect()
    }

    // ----Function related stuffs----
    pub fn define_func(&mut self, name: &ast::Identifier, params: &[ast::Param], ret: &Option<ast::Ty>) -> Result<(), SemaError> {
        if !is_snake_case(&name.value) {
            return Err(self.err(format!("function `{}` must be snake_case", name.value), name.span));
        }

        let params_ty = self.resolve_params_ty(params)?;
        let ret_ty = match ret {
            Some(ret) => self.resolve_ty(ret)?,
            None => Ty::Void
        };

        if self.funcs.contains_key(&name.value) {
            return Err(self.err(format!("function `{}` is already defined", name.value), name.span));
        }

        let id = FuncId(self.funcs.len() as u32);

        self.funcs.insert(name.value.clone(), FuncSymbol { id, params_ty, ret_ty });
        Ok(())
    }

    pub fn lookup_func(&self, name: &str) -> Option<&FuncSymbol> {
        self.funcs.get(name)
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
    pub fn define_struct(&mut self, name: &ast::Identifier) -> Result<StructId, SemaError> {
        if !is_pascal_case(&name.value) {
            return Err(self.err(format!("struct `{}` must be PascalCase", name.value), name.span));
        }

        if self.struct_ids.contains_key(&name.value) {
            return Err(self.err(format!("struct {} is already defined", name.value), name.span));
        }

        let id = StructId(self.struct_ids.len() as u32);
        self.struct_ids.insert(name.value.clone(), id);
        Ok(id)
    }

    pub fn define_struct_fields(&mut self, name: &ast::Identifier, fields: &[ast::StructField]) -> Result<StructId, SemaError> {
        let id = *self.struct_ids.get(&name.value)
            .expect("bug: getting struct id has failed, something wrong with pass 1");

        let mut map = IndexMap::new();
        for field in fields {
            if map.contains_key(&field.name.value) {
                return Err(self.err(format!("field `{}` is already defined", field.name.value), field.name.span));
            }
            map.insert(field.name.value.clone(), self.resolve_ty(&field.ty)?);
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
    pub fn define_var(&mut self, name: &ast::Identifier, ty: Ty) -> Result<VarId, SemaError> {
        if !is_snake_case(&name.value) {
            return Err(self.err(format!("variable `{}` must be snake_case", name.value), name.span));
        }

        let scope = self.scopes.last()
            .expect("bug: idk why but for some reason define_var is called without any scope");

        if scope.contains_key(&name.value) {
            return Err(self.err(format!("variable `{}` is already defined", name.value), name.span));
        }

        let id = self.alloc_local_id();
        self.scopes.last_mut()
            .expect("bug: scope vanished between the check above and here")
            .insert(name.value.clone(), VarSymbol { id, ty });
        Ok(id)
    }

    pub fn lookup_var(&self, name: &str) -> Option<&VarSymbol> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }
}
