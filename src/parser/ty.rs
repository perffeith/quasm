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

    pub(super) fn parse_type(&mut self) -> Result<Ty, ParseError> {
        match self.peek() {
            TokenKind::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.consume(TokenKind::RBracket)?;
                Ok(Ty::Array(Box::new(inner)))
            }
            TokenKind::VerBar => {
                self.advance();
                self.consume(TokenKind::Func)?;
                let params = self.parse_comma_list(TokenKind::LParen, TokenKind::RParen, "parameter type", |p| p.parse_type())?;
                let ret = if self.peek_is(TokenKind::Colon) {
                    self.advance();
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                self.consume(TokenKind::VerBar)?;
                Ok(Ty::Func { params, ret })
            }
            TokenKind::Identifier(_) => {
                let name = self.parse_identifier()?;
                Ok(Ty::Named { name })
            }
            other => Err(self.err(format!("expected type, got {:?}", other)))
        }
    }
}
