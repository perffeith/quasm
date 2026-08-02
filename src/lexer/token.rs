use logos::Logos;


#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(skip(r"//[^\r\n]*", allow_greedy = true))]
pub enum TokenKind {
    #[token("func")] Func,
    #[token("struct")] Struct,
    #[token("enum")] Enum,
    #[token("let")] Let,
    #[token("mut")] Mut,
    #[token("if")] If,
    #[token("elif")] Elif,
    #[token("else")] Else,
    #[token("for")] For,
    #[token("in")] In,
    #[token("while")] While,
    #[token("return")] Return,
    #[token("true")] True,
    #[token("false")] False,

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

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