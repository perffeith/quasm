use logos::{Lexer, Logos};

fn str_callback(lex: &mut Lexer<TokenKind>) -> Option<String> {
    let s = lex.slice();
    let inner = &s[1..s.len() - 1];

    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => result.push('"'),
            Some('\\') => result.push('\\'),
            Some('n') => result.push('\n'),
            Some('t') => result.push('\t'),
            _ => return None
        }
    }
    Some(result)
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(skip(r"//[^\r\n]*", allow_greedy = true))]
pub enum TokenKind {
    #[token("extern")] Extern,
    #[token("func")] Func,
    #[token("struct")] Struct,
    #[token("enum")] Enum,
    #[token("local")] Local,
    #[token("if")] If,
    #[token("elif")] Elif,
    #[token("else")] Else,
    #[token("for")] For,
    #[token("in")] In,
    #[token("while")] While,
    #[token("return")] Return,
    #[token("true")] True,
    #[token("false")] False,

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f32>().ok())]
    Float(f32),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().ok())]
    Int(i32),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r#""([^"\\]|\\["\\nt])*""#, str_callback)]
    StringLit(String),

    #[token(".")] Dot,
    #[token("!")] Bang,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Asterisk,
    #[token("/")] Slash,
    #[token("==")] EqEq,
    #[token("!=")] BangEq,
    #[token("<=")] LtEq,
    #[token(">=")] GtEq,
    #[token("<")] Lt,
    #[token(">")] Gt,
    #[token("=")] Eq,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token(",")] Comma,
    #[token(":")] Colon,
    #[token(";")] Semicolon,
    #[token("=>")] FatArrow,
    #[token("|")] VerBar,
    #[token("&&")] And,
    #[token("||")] Or,
    #[token("\n")] Newline,

    Eof,
    Error
}