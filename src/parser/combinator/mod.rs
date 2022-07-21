use super::{Parser, ParserError};
use std::{
    ops::{Add, BitOr, Div},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub enum ParseResult<C: Clone> {
    One(C),
    Seq(Vec<ParseResult<C>>),
}

impl<'o, C: Clone> ParseResult<C> {
    pub fn flatten(self) -> Vec<C> {
        match self {
            Self::One(c) => vec![c],
            Self::Seq(rs) => rs
                .iter()
                .map(|r| r.clone().flatten())
                .collect::<Vec<_>>()
                .concat(),
        }
    }

    pub fn len(&self) -> i64 {
        match self {
            Self::One(_) => 1,
            Self::Seq(rs) => rs
                .iter()
                .map(|r| r.len())
                .reduce(|l1, l2| l1 + l2)
                .unwrap_or(0),
        }
    }

    pub fn map<D: Clone, F>(&self, f: F) -> ParseResult<D>
    where
        F: Fn(&C) -> D + Copy,
    {
        match self {
            ParseResult::One(c) => ParseResult::One(f(c)),
            ParseResult::Seq(rs) => {
                let mut v: Vec<ParseResult<D>> = vec![];
                for r in rs {
                    v.push(r.map(f))
                }
                ParseResult::Seq(v)
            }
        }
    }
}

impl<C: Clone> From<ParseResult<C>> for ParseResult<Vec<C>> {
    fn from(orig: ParseResult<C>) -> Self {
        ParseResult::One(orig.flatten())
    }
}

pub enum CombinatorParser<C> {
    One(C),
    Seq(Vec<Rc<CombinatorParser<C>>>),
    /// Returns longest match
    Alt(Vec<Rc<CombinatorParser<C>>>),
    /// Returns first match
    Any(Vec<Rc<CombinatorParser<C>>>),
}

impl<'i: 'o, 'o, C: PartialEq + Clone> Parser<&'i [C], (&'o [C], ParseResult<C>), ParserError<C>>
    for CombinatorParser<C>
{
    fn parse(&self, input: &'i [C]) -> Result<(&'o [C], ParseResult<C>), ParserError<C>> {
        match self {
            Self::One(c) => match input {
                [h, t @ ..] => {
                    if h == c {
                        Ok((t, ParseResult::One(c.clone())))
                    } else {
                        Err(ParserError::ParseError {
                            expects: Some(c.clone()),
                            found: Some(h.clone()),
                        })
                    }
                }
                _ => Err(ParserError::ParseError {
                    expects: Some(c.clone()),
                    found: None,
                }),
            },
            Self::Seq(ps) => ps
                .iter()
                .try_fold(
                    (input, vec![]),
                    |(remaining_input, mut parse_results), parser| {
                        parser.parse(remaining_input).map(|(ri, pr)| {
                            parse_results.push(pr);
                            (ri, parse_results)
                        })
                    },
                )
                .map(|(ri, prs)| (ri, ParseResult::Seq(prs))),
            Self::Any(ps) => ps.iter().find_map(|p| p.parse(input).ok()).map_or_else(
                || {
                    ps.get(0).map(|p| p.parse(input)).unwrap_or_else(|| {
                        Err(ParserError::ParseError {
                            expects: None,
                            found: input.get(0).cloned(),
                        })
                    })
                },
                Ok,
            ),
            Self::Alt(ps) => ps
                .iter()
                .map(|p| p.parse(input))
                .reduce(
                    |r1, r2: Result<(&'o [C], ParseResult<C>), ParserError<C>>| match (&r1, &r2) {
                        (Ok((_, rr1)), Ok((_, rr2))) => {
                            if rr1.len() > rr2.len() {
                                r1
                            } else {
                                r2
                            }
                        }
                        (Err(_), Ok(_)) => r2,
                        (Ok(_), Err(_)) | (Err(_), _) => r1,
                    },
                )
                .unwrap_or_else(|| {
                    ps.get(0).map(|p| p.parse(input)).unwrap_or_else(|| {
                        Err(ParserError::ParseError {
                            expects: None,
                            found: input.get(0).cloned(),
                        })
                    })
                }),
        }
    }
}

impl<C: Clone> CombinatorParser<C> {
    pub fn new_from(c: C) -> Self {
        CombinatorParser::One(c)
    }
}

#[allow(clippy::type_complexity)]
impl<'i: 'o, 'o, C: PartialEq + Clone>
    Parser<&'i [C], (&'o [C], ParseResult<Vec<C>>), ParserError<Vec<C>>>
    for CombinatorParser<Vec<C>>
{
    fn parse(&self, input: &'i [C]) -> Result<(&'o [C], ParseResult<Vec<C>>), ParserError<Vec<C>>> {
        match self {
            Self::One(c) => c.iter().map(ToOwned::to_owned).map(CombinatorParser::One).try_fold((input, vec![]),
                    |(remaining_input, mut parse_results), parser| {
                        parser.parse(remaining_input)
                        .map_err(|err| {
                            let mut prs = parse_results.clone();
                            match err {
                                ParserError::ParseError { expects : _, found } =>
                                    ParserError::ParseError {
                                        expects: Some(c.iter().map(ToOwned::to_owned).collect()),
                                        found: found.map(|f| { prs.push(f); prs })
                                    },
                            }
                        }).map(|(ri, pr) : (&[C], ParseResult<C>)| { parse_results.append(&mut pr.flatten()); (ri, parse_results) } )
                    }
                ).map(|(ri, prs)| (ri, ParseResult::One(prs))),
            Self::Seq(ps) => ps.iter().try_fold((input, vec![]),
                    |(remaining_input, mut parse_results), parser| parser.parse(remaining_input).map(|(ri, pr)| {
                        parse_results.push(pr); (ri, parse_results)
                    } )
                ).map(|(ri, prs)| (ri, ParseResult::Seq(prs))),
            Self::Any(ps) => ps.iter().find_map(|p| p.parse(input).ok())
                .map_or_else(|| ps.get(0).map(|p| p.parse(input))
                .unwrap_or_else(|| Err(ParserError::ParseError { expects: None, found: input.get(0).cloned().map(|c| vec![c]) })), Ok),
            Self::Alt(ps) => ps.iter().map(|p| p.parse(input)).reduce(
                |r1 : Result<(&'o [C], ParseResult<Vec<C>>), ParserError<Vec<C>>>, r2 : Result<(&'o [C], ParseResult<Vec<C>>), ParserError<Vec<C>>>|
                    match (&r1, &r2) {
                        (Ok((_, rr1)), Ok((_, rr2))) => if rr1.len() > rr2.len() { r1 } else { r2 },
                        (Err(_), Ok(_)) => r2,
                        (Ok(_), Err(_)) | (Err(_), _) => r1,
                    }
                ).unwrap_or_else(|| ps.get(0).map(|p| p.parse(input))
                .unwrap_or_else(|| Err(ParserError::ParseError { expects: None, found: input.get(0).cloned().map(|c| vec!(c)) }))),
        }
    }
}

impl<'i: 'o, 'o> Parser<&'i [char], (&'o [char], ParseResult<String>), ParserError<String>>
    for CombinatorParser<Vec<char>>
{
    fn parse(
        &self,
        input: &'i [char],
    ) -> Result<(&'o [char], ParseResult<String>), ParserError<String>> {
        self.parse(input)
            .map(|(ri, pr): (&[char], ParseResult<Vec<char>>)| {
                (ri, pr.map(|s| s.iter().collect::<String>()))
            })
            .map_err(|err| match err {
                ParserError::ParseError { expects, found } => ParserError::ParseError {
                    expects: expects.map(|s| s.iter().collect::<String>()),
                    found: found.map(|s| s.iter().collect::<String>()),
                },
            })
    }
}

impl<'i: 'o, 'o, C: Clone> CombinatorParser<Vec<C>> {
    pub fn new_from_vec(s: Vec<C>) -> Self {
        CombinatorParser::One(s)
    }
}

impl CombinatorParser<Vec<char>> {
    pub fn new_from_str(s: &str) -> CombinatorParser<Vec<char>> {
        CombinatorParser::One(s.chars().collect::<Vec<char>>())
    }
}

impl<C> Add for CombinatorParser<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CombinatorParser::Seq(vec![Rc::new(self), Rc::new(rhs)])
    }
}

impl<C> BitOr for CombinatorParser<C> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        CombinatorParser::Alt(vec![Rc::new(self), Rc::new(rhs)])
    }
}

impl<C> Div for CombinatorParser<C> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        CombinatorParser::Any(vec![Rc::new(self), Rc::new(rhs)])
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::parser::combinator::ParseResult;

    use super::super::Parser;
    use super::CombinatorParser;

    #[test]
    fn simple_parse() {
        let one = CombinatorParser::Seq(vec![
            Rc::new(CombinatorParser::new_from_str("(")),
            Rc::new(CombinatorParser::new_from_str("0") | CombinatorParser::new_from_str("1")),
            Rc::new(CombinatorParser::new_from_str(")")),
        ]);

        let s = "(0)".chars().collect::<Vec<char>>();
        let result: Result<(_, ParseResult<String>), _> = one.parse(s.as_slice());
        assert!(result.is_ok());
        println!("{:?}", result);

        let s = "(1)".chars().collect::<Vec<char>>();
        let result: Result<(_, ParseResult<String>), _> = one.parse(s.as_slice());
        assert!(result.is_ok());
        println!("{:?}", result);

        let s = "(2)".chars().collect::<Vec<char>>();
        let result: Result<(_, ParseResult<String>), _> = one.parse(s.as_slice());
        assert!(result.is_err());
        println!("{:?}", result);
    }
}
