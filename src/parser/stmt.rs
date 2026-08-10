use crate::lexer::TokenKind;
use super::ast::*;
use super::Parser;
use super::ParseError;

impl Parser {
    pub(super) fn parse_top_level_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenKind::Extern => Ok(Stmt::ExternFunc(self.parse_extern_func_decl()?)),
            TokenKind::Func => Ok(Stmt::Func(self.parse_func_decl()?)),
            _ => self.parse_stmt()
        }
    }

    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenKind::Extern => Err(self.err("extern declarations are only allowed at top level")),
            TokenKind::Func => Err(self.err("nested function declarations are not allowed")),
            TokenKind::Struct => Ok(Stmt::Struct(self.parse_struct_decl()?)),
            TokenKind::Enum => Ok(Stmt::Enum(self.parse_enum_decl()?)),
            TokenKind::Local => Ok(Stmt::Local(self.parse_local_decl()?)),
            TokenKind::Return => Ok(Stmt::Return(self.parse_return()?)),
            TokenKind::If => Ok(Stmt::If(self.parse_if()?)),
            TokenKind::LBrace => Ok(Stmt::Block(self.parse_block()?)),
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

    pub(super) fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.cur_span().start;
        let stmts = self.parse_braced_list("statement", |p| p.parse_stmt())?;
        Ok(Block { stmts, span: self.span_from(start) })
    }

    fn parse_extern_func_decl(&mut self) -> Result<ExternFunc, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::Extern)?;
        let module = self.parse_identifier()?;
        self.consume(TokenKind::Slash)?;
        let symbol = self.parse_identifier()?;

        self.expect_newline("extern path")?;
        let (name, params, ret) = self.parse_func_sig()?;

        if self.peek_is(TokenKind::LBrace) {
            return Err(self.err("extern function cannot have a body"));
        }

        Ok(ExternFunc { module, symbol, name, params, ret, span: self.span_from(start) })
    }

    fn parse_func_decl(&mut self) -> Result<Func, ParseError> {
        let start = self.cur_span().start;
        let (name, params, ret) = self.parse_func_sig()?;
        let body = self.parse_block()?;
        Ok(Func { name, params, ret, body, span: self.span_from(start) })
    }

    fn parse_func_sig(&mut self) -> Result<(Identifier, Vec<Param>, Option<Ty>), ParseError> {
        self.consume(TokenKind::Func)?;
        let name = self.parse_identifier()?;
        let params = self.parse_func_params()?;
        let ret = self.parse_type_annotation()?;
        Ok((name, params, ret))
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

    fn parse_local_decl(&mut self) -> Result<Local, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::Local)?;
        let name = self.parse_identifier()?;
        let annot_ty = self.parse_type_annotation()?;
        self.consume(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        Ok(Local { name, annot_ty, value, span: self.span_from(start) })
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

    fn parse_if(&mut self) -> Result<If, ParseError> {
        let start = self.cur_span().start;
        self.consume(TokenKind::If)?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let mut elifs = Vec::new();
        while self.peek_is(TokenKind::Elif) {
            let elif_start = self.cur_span().start;
            self.advance();
            let condition = self.parse_expr()?;
            let then_block = self.parse_block()?;
            elifs.push(Elif { condition, then_block, span: self.span_from(elif_start) });
        }

        let else_block = if self.peek_is(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(If { condition, then_block, elifs, else_block, span: self.span_from(start) })
    }
}