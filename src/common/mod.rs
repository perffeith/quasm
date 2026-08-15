pub mod span;
pub mod op;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Literal {
    Int(i32),
    Float(f32),
    Bool(bool)
}