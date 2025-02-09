#![allow(dead_code)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{cut, map, map_res, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, terminated},
    Parser,
};
use nom_locate::{position, LocatedSpan};
use std::str::FromStr;

use crate::{
    error::PitaError,
    token::Token,
    value::{Decl, Id, PatternExpr, Predicate, Value},
};

type IResult<'a, O> = nom::IResult<Span<'a>, O>;
type Span<'a> = LocatedSpan<&'a str, &'static str>;

pub const KEYWORDS: &[&str] = &[
    "<-", "->", ":", ";", "else", "if", "let", "match", "do", "then",
];

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || "_!@$%^&*+=<>|".contains(c)
}
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, O, E: ParseError<Span<'a>>, F>(
    inner: F,
) -> impl Parser<Span<'a>, Output = O, Error = E>
where
    F: Parser<Span<'a>, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

fn identifier(input: Span) -> IResult<Span> {
    recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(is_identifier_char),
    ))
    .parse(input)
}

fn id_parser(input: Span) -> IResult<Id> {
    map_res(map(ws(identifier), Token::from), Id::try_from).parse(input)
}

fn number_parser(input: Span) -> IResult<Value> {
    map_res(ws(digit1), |x| i64::from_str(x.fragment()).map(Value::Int)).parse(input)
}

fn string_literal_parser(input: Span) -> IResult<Value> {
    map(string_literal, Value::Str).parse(input)
}

fn string_literal(input: Span) -> IResult<String> {
    Ok(ws(delimited(
        char('"'),
        map(
            many0(alt((
                map(tag("\\\""), |_| '"'),
                map(tag("\\n"), |_| '\n'),
                map(tag("\\t"), |_| '\t'),
                map(tag("\\r"), |_| '\r'),
                map(take_while1(|c| c != '"' && c != '\\'), |s: Span| {
                    s.chars().next().unwrap()
                }),
            ))),
            |chars| chars.into_iter().collect(),
        ),
        char('"'),
    ))
    .parse(input)?)
}

fn ctor_predicate_parser(input: Span) -> IResult<Predicate> {
    ws(map(
        pair(id_parser, many0(predicate_parser)),
        |(ctor, preds)| Predicate::Ctor(ctor, preds),
    ))
    .parse(input)
}

fn predicate_parser(input: Span) -> IResult<Predicate> {
    ws(alt((
        // Parse negative number predicates.
        map_res((tag("-"), ws(digit1)), |(_, digits)| {
            digits.parse().map(|x: i64| Predicate::Int(-x))
        }),
        // Parse positive number predicates.
        map_res(digit1, |s: Span| s.parse().map(Predicate::Int)),
        ctor_predicate_parser,
        map(id_parser, Predicate::Id),
    )))
    .parse(input)
}

fn match_parser(input: Span) -> IResult<Value> {
    map(
        (
            ws(tag("match")),
            cut((
                expr_parser,
                ws(char(':')),
                many1((
                    predicate_parser,
                    ws(tag("->")),
                    delimited(ws(char('(')), expr_parser, ws(char(')'))),
                )),
            )),
        ),
        |(_, (subject, _, patterns))| Value::Match {
            subject: Box::new(subject),
            pattern_exprs: patterns
                .into_iter()
                .map(|(predicate, _, expr)| PatternExpr { predicate, expr })
                .collect(),
        },
    )
    .parse(input)
}

fn let_parser(input: Span) -> IResult<Value> {
    map(
        (
            ws(tag("let")),
            id_parser,
            ws(char('=')),
            expr_parser,
            ws(char(':')),
            expr_parser,
        ),
        |(_, name, _, value, _, body)| Value::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        },
    )
    .parse(input)
}

fn do_line_parser(input: Span) -> IResult<DoLine> {
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

fn do_parser(input: Span) -> IResult<Value> {
    map_res(
        (
            ws(tag("do")),
            separated_list0(ws(char(';')), do_line_parser),
        ),
        |(_, lines)| convert_do_notation(&lines),
    )
    .parse(input)
}

fn lambda_parser(input: Span) -> IResult<Value> {
    map(
        (many1(id_parser), ws(tag("->")), expr_parser),
        |(params, _, body)| Value::Lambda {
            param_names: params,
            body: Box::new(body),
        },
    )
    .parse(input)
}

fn tuple_ctor_parser(input: Span) -> IResult<Value> {
    map(
        delimited(
            ws(char('(')),
            separated_list0(ws(char(',')), expr_parser),
            ws(char(')')),
        ),
        |exprs| Value::Tuple { dims: exprs },
    )
    .parse(input)
}

fn if_then_else_parser(input: Span) -> IResult<Value> {
    map(
        (
            ws(tag("if")),
            cut((
                expr_parser,
                ws(tag("then")),
                expr_parser,
                ws(tag("else")),
                expr_parser,
            )),
        ),
        |(_, (condition, _, then_expr, _, else_expr))| Value::Match {
            subject: Box::new(condition),
            pattern_exprs: vec![
                PatternExpr {
                    predicate: Predicate::Ctor("True".into(), vec![]),
                    expr: then_expr,
                },
                PatternExpr {
                    predicate: Predicate::Ctor("False".into(), vec![]),
                    expr: else_expr,
                },
            ],
        },
    )
    .parse(input)
}

fn callsite_parser(input: Span) -> IResult<Value> {
    map(many1(callsite_term_parser), |mut terms| {
        if terms.len() == 1 {
            terms.remove(0)
        } else {
            Value::Callsite {
                function: Box::new(terms.remove(0)),
                arguments: terms,
            }
        }
    })
    .parse(input)
}

fn callsite_term_parser(input: Span) -> IResult<Value> {
    ws(alt((
        string_literal_parser,
        tuple_ctor_parser,
        let_parser,
        do_parser,
        if_then_else_parser,
        match_parser,
        number_parser,
        map(id_parser, Value::Id),
    )))
    .parse(input)
}

fn decl_parser(input: Span) -> IResult<Decl> {
    map(
        (
            id_parser,
            many0(predicate_parser),
            ws(char('=')),
            expr_parser,
            ws(char(';')),
        ),
        |(name, pattern, _, body, _)| Decl {
            name,
            pattern,
            body,
        },
    )
    .parse(input)
}

pub(crate) fn program_parser(input: Span) -> IResult<Vec<Decl>> {
    terminated(many0(decl_parser), multispace0).parse(input)
}

// Helper function to convert do notation into nested expressions
fn convert_do_notation(lines: &[DoLine]) -> Result<Value, PitaError> {
    Ok(match lines {
        [] => return Err("Empty do block".into()),
        [DoLine::Expr(expr)] => expr.clone(),
        [DoLine::Let(name, value), rest @ ..] => Value::Let {
            name: name.clone(),
            value: Box::new(value.clone()),
            body: Box::new(convert_do_notation(rest)?),
        },
        [DoLine::Bind(name, expr), rest @ ..] => Value::Callsite {
            function: Box::new(Value::Id(">>=".into())),
            arguments: vec![
                expr.clone(),
                Value::Lambda {
                    param_names: vec![name.clone()],
                    body: Box::new(convert_do_notation(rest)?),
                },
            ],
        },
        [DoLine::Expr(_), ..] => return Err("Expression in middle of do block".into()),
    })
}

#[derive(Debug, Clone)]
enum DoLine {
    Bind(Id, Value),
    Let(Id, Value),
    Expr(Value),
}

fn expr_parser(input: Span) -> IResult<Value> {
    ws(alt((
        match_parser,
        number_parser,
        map(id_parser, Value::Id),
        // ... other expression types
    )))
    .parse(input)
}
