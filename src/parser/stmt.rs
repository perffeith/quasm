use crate::lexer::TokenKind;
use super::ast::*;
use super::Parser;
use super::ParseError;

impl Parser {
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenKind::Func => Ok(Stmt::Func(self.parse_func_decl()?)),
            TokenKind::Struct => Ok(Stmt::Struct(self.parse_struct_decl()?)),
            TokenKind::Let => Ok(Stmt::Let(self.parse_let_stmt()?)),
            TokenKind::Type => Ok(Stmt::Type(self.parse_type_decl()?)),
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    fn parse_func_decl(&mut self) -> Result<Func, ParseError> {
        self.consume(TokenKind::Func)?;
        let name = self.parse_identifier()?;
        let params = self.parse_func_params()?;
        let ret = if self.peek_is(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Func { name, params, ret, body })
    }

    fn parse_func_params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "parameter", |p| {
            let param = p.parse_param()?;
            if param.ty.is_none() {
                return Err(p.err("expected type annotation for parameter"));
            }
            Ok(param)
        })
    }

    fn parse_let_stmt(&mut self) -> Result<Let, ParseError> {
        self.consume(TokenKind::Let)?;
        let is_mut = self.peek_is(TokenKind::Mut);
        if is_mut {
            self.advance();
        }
        let name = self.parse_identifier()?;
        let annot_ty = self.parse_type_annotation()?;
        self.consume(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        Ok(Let { name, is_mut, annot_ty, value })
    }

    fn parse_type_decl(&mut self) -> Result<Type, ParseError> {
        self.consume(TokenKind::Type)?;
        let name = self.parse_identifier()?;
        let ty_params = self.parse_ty_params()?;
        let variants = self.parse_braced_list("variant", |p| p.parse_type_variant())?;
        Ok(Type { name, ty_params, variants })
    }

    fn parse_type_variant(&mut self) -> Result<TypeVariant, ParseError> {
        let name = self.parse_identifier()?;

        let ty_fields = if self.peek_is(TokenKind::LParen) {
            self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "type field", |p| p.parse_type())?
        } else {
            Vec::new()
        };

        Ok(TypeVariant { name, ty_fields })
    }

    fn parse_struct_decl(&mut self) -> Result<Struct, ParseError> {
        self.consume(TokenKind::Struct)?;
        let name = self.parse_identifier()?;
        let ty_params = self.parse_ty_params()?;
        let fields = self.parse_braced_list("field", |p| p.parse_struct_field())?;
        Ok(Struct { name, ty_params, fields })
    }

    fn parse_struct_field(&mut self) -> Result<StructField, ParseError> {
        let name = self.parse_identifier()?;
        self.consume(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok(StructField { name, ty })
    }
}
