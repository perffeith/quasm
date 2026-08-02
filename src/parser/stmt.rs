use crate::lexer::TokenKind;
use super::ast::*;
use super::Parser;
use super::ParseError;

impl Parser {
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenKind::Func => Ok(Stmt::Func(self.parse_func_decl()?)),
            TokenKind::Struct => Ok(Stmt::Struct(self.parse_struct_decl()?)),
            TokenKind::Enum => Ok(Stmt::Enum(self.parse_enum_decl()?)),
            TokenKind::Let => Ok(Stmt::Let(self.parse_let_stmt()?)),
            TokenKind::Return => Ok(Stmt::Return(self.parse_return()?)),
            TokenKind::If => Ok(Stmt::If(self.parse_if_stmt()?)),
            _ => {
                let expr = self.parse_expr()?;
                if self.peek_is(TokenKind::Eq) {
                    let start = expr.span().start;
                    self.advance();
                    let value = self.parse_expr()?;
                    Ok(Stmt::Assign(Assign { target: expr, value, span: self.span_from(start) }))
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_func_decl(&mut self) -> Result<Func, ParseError> {
        let start = self.cur_span().start;
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
        Ok(Func { name, params, ret, body, span: self.span_from(start) })
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

    fn parse_struct_decl(&mut self) -> Result<Struct, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::Struct)?;
        let name = self.parse_identifier()?;
        let fields = self.parse_braced_list("field", |p| p.parse_struct_field())?;
        Ok(Struct { name, fields, span: self.span_from(start) })
    }

    fn parse_struct_field(&mut self) -> Result<StructField, ParseError> {
        let start = self.cur_span().start;
        let name = self.parse_identifier()?;
        self.consume(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok(StructField { name, ty, span: self.span_from(start) })
    }
    
    fn parse_enum_decl(&mut self) -> Result<Enum, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::Enum)?;
        let name = self.parse_identifier()?;
        let variants = self.parse_braced_list("variant", |p| p.parse_enum_variant())?;
        Ok(Enum { name, variants, span: self.span_from(start) })
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParseError> {
        let start = self.cur_span().start;
        let name = self.parse_identifier()?;

        let ty_fields = if self.peek_is(TokenKind::LParen) {
            self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "enum field", |p| p.parse_type())?
        } else {
            Vec::new()
        };

        Ok(EnumVariant { name, ty_fields, span: self.span_from(start) })
    }

    fn parse_let_stmt(&mut self) -> Result<Let, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::Let)?;
        let is_mut = self.peek_is(TokenKind::Mut);
        if is_mut {
            self.advance();
        }
        let name = self.parse_identifier()?;
        let annot_ty = self.parse_type_annotation()?;
        self.consume(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        Ok(Let { name, is_mut, annot_ty, value, span: self.span_from(start) })
    }

    fn parse_return(&mut self) -> Result<Return, ParseError> {
        self.consume(TokenKind::Return)?;

        let span = self.cur_span();
        let value = match self.peek() {
            TokenKind::Newline | TokenKind::RBrace => None,
            _ => Some(self.parse_expr()?)
        };

        let span = match &value {
            Some(expr) => expr.span(),
            None => span
        };

        Ok(Return { value, span })
    }

    fn parse_if_stmt(&mut self) -> Result<If, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::If)?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_branch = match self.peek() {
            TokenKind::Elif => Some(Else::ElseIf(Box::new(self.parse_if_stmt()?))),
            TokenKind::Else => {
                self.advance();
                Some(Else::Block(self.parse_block()?))
            }
            _ => None
        };

        Ok(If { condition, then_block, else_branch, span: self.span_from(start) })
    }
}