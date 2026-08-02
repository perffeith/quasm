#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOpKind {
    // arithmetic
    Add, Sub, Mul, Div,
    // comparison
    Lt, Gt, LtEq, GtEq,
    // equality
    EqEq, NotEq,
    // logical
    And, Or
}

impl std::fmt::Display for BinOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinOpKind::Add => "+",
            BinOpKind::Sub => "-",
            BinOpKind::Mul => "*",
            BinOpKind::Div => "/",
            BinOpKind::Lt => "<",
            BinOpKind::Gt => ">",
            BinOpKind::LtEq => "<=",
            BinOpKind::GtEq => ">=",
            BinOpKind::EqEq => "==",
            BinOpKind::NotEq => "!=",
            BinOpKind::And => "&&",
            BinOpKind::Or => "||"
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpKind {
    Neg,
    Not
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool)
}