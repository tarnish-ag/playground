pub(self) mod combinator;
pub(super) mod tree;

pub trait Parser<I, O, E> {
    fn parse(&self, input: I) -> Result<O, E>;
}

#[derive(Debug, Clone)]
pub enum ParserError<C> {
    ParseError {
        expects: Option<C>,
        found: Option<C>,
    },
}

impl<C> ParserError<C> {
    pub fn map<F, D>(self, op: F) -> ParserError<D>
    where
        F: Fn(C) -> D,
    {
        match self {
            Self::ParseError { expects, found } => ParserError::ParseError {
                expects: expects.map(&op),
                found: found.map(&op),
            },
        }
    }
}
