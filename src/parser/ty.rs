use crate::lexer::TokenKind;
use super::ast::*;
use super::Parser;
use super::ParseError;

impl Parser {
    pub(super) fn parse_type_annotation(&mut self) -> Result<Option<Ty>, ParseError> {
        if !self.peek_is(TokenKind::Colon) {
            return Ok(None);
        }
        self.advance();
        Ok(Some(self.parse_type()?))
    }

    // type parameter list of type/struct declaration
    pub(super) fn parse_ty_params(&mut self) -> Result<Vec<Identifier>, ParseError> {
        if !self.peek_is(TokenKind::LParen) {
            return Ok(Vec::new());
        }
        let params = self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "type parameter", |p| p.parse_identifier())?;
        if params.is_empty() {
            return Err(self.err("type parameter list cannot be empty"));
        }
        Ok(params)
    }

    pub(super) fn parse_type(&mut self) -> Result<Ty, ParseError> {
        match self.peek() {
            TokenKind::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.consume(TokenKind::RBracket)?;
                Ok(Ty::Array(Box::new(inner)))
            }
            TokenKind::Func => {
                self.advance();
                let params = self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "parameter type", |p| p.parse_type())?;
                let ret = if self.peek_is(TokenKind::Arrow) {
                    self.advance();
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                Ok(Ty::Func { params, ret })
            }
            TokenKind::Identifier(_) => {
                let name = self.parse_identifier()?;
                let args = if self.peek_is(TokenKind::LParen) {
                    let args = self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "type argument", |p| p.parse_type())?;
                    if args.is_empty() {
                        return Err(self.err("type argument list cannot be empty"));
                    }
                    args
                } else {
                    Vec::new()
                };
                Ok(Ty::Named { name, args })
            }
            other => Err(self.err(format!("expected type, got {:?}", other)))
        }
    }
}
