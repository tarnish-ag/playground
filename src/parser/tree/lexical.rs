use super::val::Val;

pub enum Lexical {
    Const(Val),
    Id(String),
}
