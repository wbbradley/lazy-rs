#![allow(dead_code)]
use crate::id::Id;

#[derive(Debug)]
pub enum RuntimeError {
    UnresolvedSymbol(Id),
    InvalidDecl(String),
    InvalidCallsite(String),
    NoMatch(String),
    MatchTypeError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UnresolvedSymbol(id) => {
                write!(f, "pita runtime error: unresolved symbol: {}", id)
            }
            RuntimeError::InvalidDecl(msg) => {
                write!(f, "pita runtime error: invalid declaration: {msg}")
            }
            RuntimeError::InvalidCallsite(msg) => {
                write!(f, "pita runtime error: invalid callsite: {msg}")
            }
            RuntimeError::NoMatch(msg) => write!(f, "pita runtime error: no match: {msg}"),
            RuntimeError::MatchTypeError(msg) => {
                write!(f, "pita runtime error: match type error: {msg}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
