#![allow(dead_code)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, terminated},
    IResult,
    Parser,
};

use crate::{error::Error, value::Value};

const KEYWORDS: &[&str] = &["->", ":", "else", "if", "let", "match", "do", "then"];

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || "_!@$%^&*+=<>|".contains(c)
}
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(is_identifier_char),
    ))
    .parse(input)
}

fn id_parser(input: &str) -> IResult<&str, Value> {
    map_res(ws(identifier), |s: &str| {
        if !s.chars().next().unwrap().is_uppercase() && !KEYWORDS.contains(&s) {
            Ok(Value::id(s))
        } else {
            Err(Error::from(format!("invalid identifier: {}", s)))
        }
    })
    .parse(input)
}

fn ctor_parser(input: &str) -> IResult<&str, term::Ctor> {
    map(ws(identifier), |s: &str| {
        if s.chars().next().unwrap().is_uppercase() && !KEYWORDS.contains(&s) {
            term::Ctor {
                name: s.to_string(),
            }
        } else {
            panic!("Invalid constructor: {}", s)
        }
    })
    .parse(input)
}

fn number_parser(input: &str) -> IResult<&str, term::Expr> {
    map(ws(digit1), |s: &str| {
        term::Expr::Literal(term::Literal::Int(s.parse().unwrap()))
    })
    .parse(input)
}

fn string_literal_parser(input: &str) -> IResult<&str, term::Expr> {
    map(string_literal, |s| {
        term::Expr::Literal(term::Literal::Str(s))
    })
    .parse(input)
}

fn string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            many0(alt((
                map(tag("\\\""), |_| '"'),
                map(tag("\\n"), |_| '\n'),
                map(tag("\\t"), |_| '\t'),
                map(tag("\\r"), |_| '\r'),
                map(take_while1(|c| c != '"' && c != '\\'), |s: &str| {
                    s.chars().next().unwrap()
                }),
            ))),
            |chars| chars.into_iter().collect(),
        ),
        char('"'),
    )
    .parse(input)
}

fn predicate_parser(input: &str) -> IResult<&str, term::Predicate> {
    alt((
        map(ws(digit1), |s: &str| {
            term::Predicate::Int(s.parse().unwrap())
        }),
        map(
            (ctor_parser, many0(ws(predicate_parser))),
            |(ctor, args)| term::Predicate::Ctor(ctor, args),
        ),
        map(id_parser, term::Predicate::Id),
    ))
    .parse(input)
}

fn match_parser(input: &str) -> IResult<&str, term::Expr> {
    map(
        (
            ws(tag("match")),
            expr_parser,
            ws(char(':')),
            many1((
                predicate_parser,
                ws(tag("->")),
                delimited(ws(char('(')), expr_parser, ws(char(')'))),
            )),
        ),
        |(_, subject, _, patterns)| {
            term::Expr::Match(term::Match {
                subject: Box::new(subject),
                pattern_exprs: patterns
                    .into_iter()
                    .map(|(predicate, _, expr)| term::PatternExpr { predicate, expr })
                    .collect(),
            })
        },
    )
    .parse(input)
}

fn let_parser(input: &str) -> IResult<&str, term::Expr> {
    map(
        (
            ws(tag("let")),
            id_parser,
            ws(char('=')),
            expr_parser,
            ws(char(':')),
            expr_parser,
        ),
        |(_, name, _, value, _, body)| {
            term::Expr::Let(term::Let {
                name,
                value: Box::new(value),
                body: Box::new(body),
            })
        },
    )
    .parse(input)
}

fn do_line_parser(input: &str) -> IResult<&str, DoLine> {
    alt((
        // bind syntax: x <- expr
        map((id_parser, ws(tag("<-")), expr_parser), |(id, _, expr)| {
            DoLine::Bind(id, expr)
        }),
        // let syntax: let x = expr
        map(
            (ws(tag("let")), id_parser, ws(char('=')), expr_parser),
            |(_, id, _, expr)| DoLine::Let(id, expr),
        ),
        // expression by itself
        map(expr_parser, DoLine::Expr),
    ))
    .parse(input)
}

fn do_parser(input: &str) -> IResult<&str, term::Expr> {
    map(
        (
            ws(tag("do")),
            separated_list0(ws(char(',')), do_line_parser),
        ),
        |(_, lines)| convert_do_notation(&lines),
    )
    .parse(input)
}

fn lambda_parser(input: &str) -> IResult<&str, term::Expr> {
    map(
        (many1(id_parser), ws(tag("->")), expr_parser),
        |(params, _, body)| {
            term::Expr::Lambda(term::Lambda {
                param_names: params,
                body: Box::new(body),
            })
        },
    )
    .parse(input)
}

fn tuple_ctor_parser(input: &str) -> IResult<&str, term::Expr> {
    map(
        delimited(
            ws(char('(')),
            separated_list0(ws(char(',')), expr_parser),
            ws(char(')')),
        ),
        |exprs| term::Expr::TupleCtor(term::TupleCtor { dims: exprs }),
    )
    .parse(input)
}

fn if_then_else_parser(input: &str) -> IResult<&str, term::Expr> {
    map(
        (
            ws(tag("if")),
            expr_parser,
            ws(tag("then")),
            expr_parser,
            ws(tag("else")),
            expr_parser,
        ),
        |(_, condition, _, then_expr, _, else_expr)| {
            term::Expr::Match(term::Match {
                subject: Box::new(condition),
                pattern_exprs: vec![
                    term::PatternExpr {
                        predicate: term::Predicate::Ctor(
                            term::Ctor {
                                name: "True".to_string(),
                            },
                            vec![],
                        ),
                        expr: then_expr,
                    },
                    term::PatternExpr {
                        predicate: term::Predicate::Ctor(
                            term::Ctor {
                                name: "False".to_string(),
                            },
                            vec![],
                        ),
                        expr: else_expr,
                    },
                ],
            })
        },
    )
    .parse(input)
}

fn callsite_parser(input: &str) -> IResult<&str, term::Expr> {
    map(many1(callsite_term_parser), |mut terms| {
        if terms.len() == 1 {
            terms.remove(0)
        } else {
            term::Expr::Callsite(term::Callsite {
                function: Box::new(terms.remove(0)),
                arguments: terms,
            })
        }
    })
    .parse(input)
}

fn callsite_term_parser(input: &str) -> IResult<&str, term::Expr> {
    ws(alt((
        string_literal_parser,
        tuple_ctor_parser,
        let_parser,
        do_parser,
        if_then_else_parser,
        match_parser,
        number_parser,
        map(id_parser, term::Expr::Id),
        map(ctor_parser, term::Expr::Ctor),
    )))
    .parse(input)
}

fn decl_parser(input: &str) -> IResult<&str, term::Decl> {
    map(
        (
            many1(predicate_parser),
            ws(char('=')),
            expr_parser,
            ws(char(';')),
        ),
        |(mut preds, _, body, _)| term::Decl {
            name: match preds.remove(0) {
                term::Predicate::Id(id) => id,
                _ => panic!("Declaration must start with an identifier"),
            },
            pattern: preds,
            body,
        },
    )
    .parse(input)
}

pub(crate) fn program_parser(input: &str) -> IResult<&str, Vec<term::Decl>> {
    terminated(many0(decl_parser), multispace0).parse(input)
}

// Helper function to convert do notation into nested expressions
fn convert_do_notation(lines: &[DoLine]) -> term::Expr {
    match lines {
        [] => panic!("Empty do block"),
        [DoLine::Expr(expr)] => expr.clone(),
        [DoLine::Let(name, value), rest @ ..] => term::Expr::Let(term::Let {
            name: name.clone(),
            value: Box::new(value.clone()),
            body: Box::new(convert_do_notation(rest)),
        }),
        [DoLine::Bind(name, expr), rest @ ..] => term::Expr::Callsite(term::Callsite {
            function: Box::new(term::Expr::Id(term::Id {
                name: ">>=".to_string(),
            })),
            arguments: vec![
                expr.clone(),
                term::Expr::Lambda(term::Lambda {
                    param_names: vec![name.clone()],
                    body: Box::new(convert_do_notation(rest)),
                }),
            ],
        }),
        [DoLine::Expr(_), ..] => panic!("Expression in middle of do block"),
    }
}

#[derive(Debug, Clone)]
enum DoLine {
    Bind(term::Id, term::Expr),
    Let(term::Id, term::Expr),
    Expr(term::Expr),
}

fn expr_parser(input: &str) -> IResult<&str, term::Expr> {
    alt((
        match_parser,
        number_parser,
        map(id_parser, term::Expr::Id),
        // ... other expression types
    ))
    .parse(input)
}
