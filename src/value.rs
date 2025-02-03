#![allow(dead_code)]
use std::{fmt::Debug, rc::Rc};

use crate::env::Env;

#[derive(Debug, Clone)]
pub enum Predicate {
    Id(String),
    Int(i64),
    Tuple(Vec<Predicate>),
    Ctor(String, Vec<Predicate>),
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub name: String,
    pub pattern: Vec<Predicate>,
    pub body: Value,
}

#[derive(Debug, Clone)]
pub struct PatternExpr {
    pub predicate: Predicate,
    pub expr: Value,
}

// Runtime values
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Str(Rc<String>),
    Null,
    Lambda {
        param_names: Vec<String>,
        body: Rc<Value>,
    },
    Id(Rc<String>),
    Match {
        subject: Rc<Value>,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        function: Rc<Value>,
        arguments: Vec<Value>,
    },
    Tuple {
        dims: Vec<Value>,
    },
    Thunk {
        env: Env,
        expr: Rc<Value>,
        memoized: Option<Rc<Value>>,
    },
    Builtin(Rc<dyn Fn(Vec<Value>) -> Value>),
    Let {
        name: String,
        value: Rc<Value>,
        body: Rc<Value>,
    },
    Ctor {
        name: String,
        dims: Vec<Value>,
    },
}

impl Value {
    pub(crate) fn builtin(f: Rc<dyn Fn(Vec<Value>) -> Value>) -> Self {
        Self::Builtin(f)
    }
    pub(crate) fn id(name: impl std::ops::AsRef<str>) -> Self {
        Self::Id(Rc::new(name.to_string()))
    }
}
impl Debug for Value {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
