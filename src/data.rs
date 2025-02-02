#![allow(dead_code)]
use std::{collections::HashMap, fmt::Debug, rc::Rc};

// Core types
#[derive(Debug, Clone)]
pub struct Id {
    name: String,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    name: String,
}

#[derive(Debug, Clone)]
pub enum Predicate {
    Id(Id),
    Int(i32),
    Tuple(Vec<Predicate>),
    Ctor(Ctor, Vec<Predicate>),
}

#[derive(Debug, Clone)]
pub struct Decl {
    name: Id,
    pattern: Vec<Predicate>,
    body: Expr,
}

#[derive(Debug, Clone)]
pub struct PatternExpr {
    predicate: Predicate,
    expr: Expr,
}

#[derive(Debug, Clone)]
pub struct Match {
    subject: Box<Expr>,
    pattern_exprs: Vec<PatternExpr>,
}

#[derive(Debug, Clone)]
pub struct TupleCtor {
    dims: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Let {
    name: Id,
    value: Box<Expr>,
    body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    param_names: Vec<Id>,
    body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Callsite {
    function: Box<Expr>,
    arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lambda(Lambda),
    Literal(LiteralValue),
    Id(Id),
    Match(Match),
    Callsite(Callsite),
    TupleCtor(TupleCtor),
    Let(Let),
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i32),
    Str(String),
    None,
}

// Runtime values
#[derive(Clone)]
pub enum Value {
    Lambda(Rc<Lambda>),
    Literal(LiteralValue),
    Id(Id),
    Match(Rc<Match>),
    Callsite(Rc<Callsite>),
    TupleCtorInstance(TupleCtorInstance),
    Thunk(Thunk),
    CtorInstance(CtorInstance),
    Function(Rc<dyn Fn(Vec<Value>) -> Value>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Lambda(l) => write!(f, "Lambda({:?})", l),
            Value::Literal(literal) => write!(f, "Literal({:?})", literal),
            Value::Id(id) => write!(f, "Id({:?})", id),
            Value::Match(m) => write!(f, "Match({:?})", m),
            Value::Callsite(c) => write!(f, "Callsite({:?})", c),
            Value::TupleCtorInstance(t) => write!(f, "TupleCtorInstance({:?})", t),
            Value::Thunk(t) => write!(f, "Thunk({:?})", t),
            Value::CtorInstance(c) => write!(f, "CtorInstance({:?})", c),
            Value::Function(_) => write!(f, "Function(..)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Thunk {
    env: Rc<Env>,
    expr: Box<Value>,
    memoized: Option<Box<Value>>,
}

#[derive(Debug, Clone)]
pub struct CtorInstance {
    ctor: Ctor,
    dims: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct TupleCtorInstance {
    dims: Vec<Value>,
}

// Environment
#[derive(Debug, Clone)]
pub struct Env {
    globals: HashMap<String, Value>,
    locals: HashMap<String, Value>,
}

impl Env {
    pub fn new(globals: HashMap<String, Value>) -> Self {
        Self {
            globals,
            locals: HashMap::new(),
        }
    }

    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.locals.contains_key(symbol) || self.globals.contains_key(symbol)
    }

    pub fn get_symbol(&self, symbol: &str) -> Option<Value> {
        self.locals
            .get(symbol)
            .cloned()
            .or_else(|| self.globals.get(symbol).cloned())
    }

    pub fn add_global_symbol(&mut self, symbol: String, value: Value) {
        self.globals.insert(symbol, value);
    }

    pub fn add_symbol(&mut self, symbol: String, value: Value) {
        self.locals.insert(symbol, value);
    }
}
