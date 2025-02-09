#![allow(dead_code)]
use crate::value::Id;

#[derive(Debug)]
pub enum RuntimeError {
    UnresolvedSymbol(Id),
    InvalidDecl(String),
    InvalidCallsite(String),
    NoMatch(String),
    MatchTypeError(String),
}
