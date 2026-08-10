pub mod ast;
mod expr;
mod stmt;
mod ty;

use crate::common::span::{Pos, Span};
use crate::lexer::{Token, TokenKind};
use ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    Parser::new(tokens).parse_program()
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !self.peek_is(TokenKind::Eof) {
            stmts.push(self.parse_top_level_stmt()?);
            self.expect_newline("statement")?;
        }

        Ok(Program { stmts })
    }

    fn peek(&self) -> TokenKind {
        self.tokens.get(self.pos).map(|t| t.kind.clone()).unwrap_or(TokenKind::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn peek_is(&self, kind: TokenKind) -> bool {
        self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof) == &kind
    }

    fn peek_until(&self, kind: TokenKind) -> bool {
        !self.peek_is(TokenKind::Eof) && !self.peek_is(kind)
    }

    fn skip_newlines(&mut self) {
        while self.peek_is(TokenKind::Newline) {
            self.advance();
        }
    }

    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let k = self.peek();
        if k == expected {
            self.advance();
            Ok(())
        } else {
            Err(self.err(format!("expected {:?}, got {:?}", expected, k)))
        }
    }

    pub(super) fn cur_span(&self) -> Span {
        self.tokens.get(self.pos.min(self.tokens.len() - 1)).unwrap().span
    }

    fn prev_end(&self) -> Pos {
        self.tokens[self.pos.saturating_sub(1)].span.end
    }

    pub(super) fn span_from(&self, start: Pos) -> Span {
        Span { start, end: self.prev_end() }
    }

    fn err(&self, msg: impl Into<String>) -> ParseError {
        ParseError { message: msg.into(), span: self.cur_span() }
    }

    fn expect_newline(&mut self, context: &str) -> Result<(), ParseError> {
        match self.peek() {
            TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof => {}
            other => return Err(self.err(format!("expected newline after {}, got {:?}", context, other)))
        }
        self.skip_newlines();
        Ok(())
    }

    // Parses comma list with singleline/multiline enforcement
    // Multiline mode is triggered when '(' is immediately followed by a newline
    fn parse_comma_list<T, F>(&mut self, open: TokenKind, close: TokenKind, label: &str, mut parse_item: F) -> Result<Vec<T>, ParseError>
    where
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        self.consume(open)?;
        let multiline = self.peek_is(TokenKind::Newline);
        self.skip_newlines();
        let mut items = Vec::new();

        while self.peek_until(close.clone()) {
            items.push(parse_item(self)?);

            if !self.peek_is(TokenKind::Comma) {
                if multiline && !self.peek_is(TokenKind::Newline) {
                    return Err(self.err(format!("closing {:?} must be on its own line in multiline style", close)));
                }
                break;
            }
            self.consume(TokenKind::Comma)?;
            if multiline {
                self.skip_newlines();
            } else if self.peek_is(TokenKind::Newline) {
                return Err(self.err(format!("{label}s must all be on the same line; use a newline after '(' for multiline style")));
            }
        }

        self.skip_newlines();
        if self.peek_is(TokenKind::Comma) {
            return Err(self.err(format!("comma must follow a {label} on the same line, not precede it on the next line")));
        }
        self.consume(close)?;
        Ok(items)
    }

    // Parses a {...} body of newline separated items
    fn parse_braced_list<T, F>(&mut self, label: &str, mut parse_item: F) -> Result<Vec<T>, ParseError>
    where
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        self.consume(TokenKind::LBrace)?;
        self.skip_newlines();
        let mut items = Vec::new();

        while self.peek_until(TokenKind::RBrace) {
            items.push(parse_item(self)?);
            self.expect_newline(label)?;
        }

        self.consume(TokenKind::RBrace)?;
        Ok(items)
    }
}