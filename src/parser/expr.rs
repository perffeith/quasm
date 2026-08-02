use crate::common::ast::{Literal, BinOpKind, UnaryOpKind};
use crate::common::span::{Pos, Span};
use crate::lexer::TokenKind;
use super::ast::*;
use super::Parser;
use super::ParseError;

fn new_binary(op: BinOpKind, left: Expr, right: Expr) -> Expr {
    let span = left.span.to(right.span);
    Expr { kind: ExprKind::BinaryOp(BinaryOp {op, left: Box::new(left), right: Box::new(right)}), span }
}

impl Parser {
    fn literal_expr(&mut self, literal: Literal) -> Expr {
        let span = self.cur_span();
        self.advance();
        Expr { kind: ExprKind::Literal(literal), span }
    }

    pub(super) fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.cur_span().start;
        let stmts = self.parse_braced_list("statement", |p| p.parse_stmt())?;
        Ok(Block { stmts, span: self.span_from(start) })
    }

    pub(super) fn parse_identifier(&mut self) -> Result<Identifier, ParseError> {
        match self.peek() {
            TokenKind::Identifier(value) => {
                let span = self.cur_span();
                self.advance();
                Ok(Identifier { value, span })
            }
            other => Err(self.err(format!("expected identifier, got {:?}", other)))
        }
    }

    pub(super) fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = self.parse_identifier()?;
        let ty = self.parse_type_annotation()?;
        Ok(Param { name, ty })
    }

    pub(super) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;

        while self.peek_is(TokenKind::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = new_binary(BinOpKind::Or, left, right);
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;

        while self.peek_is(TokenKind::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = new_binary(BinOpKind::And, left, right);
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                TokenKind::EqEq => BinOpKind::EqEq,
                TokenKind::BangEq => BinOpKind::NotEq,
                _ => break
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = new_binary(op, left, right);
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                TokenKind::Lt => BinOpKind::Lt,
                TokenKind::Gt => BinOpKind::Gt,
                TokenKind::LtEq => BinOpKind::LtEq,
                TokenKind::GtEq => BinOpKind::GtEq,
                _ => break
            };
            self.advance();
            let right = self.parse_additive()?;
            left = new_binary(op, left, right);
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOpKind::Add,
                TokenKind::Minus => BinOpKind::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = new_binary(op, left, right);
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                TokenKind::Asterisk => BinOpKind::Mul,
                TokenKind::Slash => BinOpKind::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = new_binary(op, left, right);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let op = match self.peek() {
            TokenKind::Minus => UnaryOpKind::Neg,
            TokenKind::Bang => UnaryOpKind::Not,
            _ => return self.parse_postfix(),
        };

        let start = self.cur_span().start;
        self.advance();
        let operand = self.parse_unary()?;
        let span = Span { start, end: operand.span.end };
        Ok(Expr { kind: ExprKind::UnaryOp { op, operand: Box::new(operand) }, span })
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                TokenKind::Dot => {
                    self.advance();
                    let name = self.parse_identifier()?;
                    if self.peek_is(TokenKind::LParen) {
                        let args = self.parse_call_args()?;
                        let span = self.span_from(expr.span.start);
                        expr = Expr { kind: ExprKind::DotCall { base: Box::new(expr), callee: name, args }, span };
                    } else {
                        let span = expr.span.to(name.span);
                        expr = Expr { kind: ExprKind::DotAccess { base: Box::new(expr), name }, span };
                    }
                }
                TokenKind::LParen => {
                    let args = self.parse_call_args()?;
                    let span = self.span_from(expr.span.start);
                    expr = Expr { kind: ExprKind::Call(Call { callee: Box::new(expr), args }), span };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.consume(TokenKind::RBracket)?;
                    let span = self.span_from(expr.span.start);
                    expr = Expr { kind: ExprKind::Index { base: Box::new(expr), index: Box::new(index) }, span };
                }
                _ => break
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            TokenKind::Int(value) => {
                Ok(self.literal_expr(Literal::Int(value)))
            }
            TokenKind::Float(value) => {
                Ok(self.literal_expr(Literal::Float(value)))
            }
            TokenKind::True => {
                Ok(self.literal_expr(Literal::Bool(true)))
            }
            TokenKind::False => {
                Ok(self.literal_expr(Literal::Bool(false)))
            }
            TokenKind::LParen => {
                self.advance();
                self.skip_newlines();
                let expr = self.parse_expr()?;
                self.skip_newlines();
                self.consume(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                let start = self.cur_span().start;
                let elems = self.parse_comma_list(TokenKind::LBracket, TokenKind::RBracket, "element", |p| p.parse_expr())?;
                Ok(Expr { kind: ExprKind::Array(elems), span: self.span_from(start) })
            }
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                let span = block.span;
                Ok(Expr { kind: ExprKind::Block(block), span })
            }
            TokenKind::If => {
                let start = self.cur_span().start;
                self.advance();
                self.parse_if_expr(start)
            }
            TokenKind::VerBar | TokenKind::Or => {
                self.parse_closure()
            }
            TokenKind::Identifier(_) => {
                let ident = self.parse_identifier()?;
                let span = ident.span;
                Ok(Expr { kind: ExprKind::Identifier(ident), span })
            }
            other => Err(self.err(format!("expected expression, got {:?}", other)))
        }
    }

    fn parse_if_expr(&mut self, start: Pos) -> Result<Expr, ParseError> {
        // start is the position of the already consumed if/elif keyword
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_branch = match self.peek() {
            TokenKind::Elif => {
                let elif_start = self.cur_span().start;
                self.advance();
                Some(Box::new(self.parse_if_expr(elif_start)?))
            }
            TokenKind::Else => {
                self.advance();
                let block = self.parse_block()?;
                let span = block.span;
                Some(Box::new(Expr { kind: ExprKind::Block(block), span }))
            }
            _ => None
        };

        let end = else_branch.as_ref().map(|e| e.span.end).unwrap_or(then_block.span.end);
        Ok(Expr {
            kind: ExprKind::If { condition: Box::new(condition), then_block, else_branch },
            span: Span { start, end }
        })
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "argument", |p| p.parse_expr())
    }

    fn parse_closure(&mut self) -> Result<Expr, ParseError> {
        let start = self.cur_span().start;

        // || lexes as the Or token, so an empty parameter list arrives as a single token
        let params = if self.peek_is(TokenKind::Or) {
            self.advance();
            Vec::new()
        } else {
            self.parse_comma_list(TokenKind::VerBar, TokenKind::VerBar, "parameter", |p| p.parse_param())?
        };

        let ret = if self.peek_is(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(TokenKind::FatArrow)?;
        let body = self.parse_expr()?;

        let span = Span { start, end: body.span.end };
        Ok(Expr { kind: ExprKind::Closure { params, ret, body: Box::new(body) }, span })
    }
}
