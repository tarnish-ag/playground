pub enum Val {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl From<i64> for Val {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for Val {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for Val {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<String> for Val {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
