#![allow(dead_code)]
use std::{collections::HashMap, fmt::Debug, rc::Rc};

// Core types
#[derive(Debug, Clone)]
pub struct Id {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub name: String,
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
    pub name: Id,
    pub pattern: Vec<Predicate>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct PatternExpr {
    pub predicate: Predicate,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub subject: Box<Expr>,
    pub pattern_exprs: Vec<PatternExpr>,
}

#[derive(Debug, Clone)]
pub struct TupleCtor {
    pub dims: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: Id,
    pub value: Box<Expr>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub param_names: Vec<Id>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Callsite {
    pub function: Box<Expr>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lambda(Lambda),
    Literal(LiteralValue),
    Id(Id),
    Match(Match),
    Callsite(Callsite),
    Ctor(Ctor),
    TupleCtor(TupleCtor),
    Let(Let),
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i64),
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
    pub env: Rc<Env>,
    pub expr: Box<Value>,
    pub memoized: Option<Box<Value>>,
}

#[derive(Debug, Clone)]
pub struct CtorInstance {
    pub ctor: Ctor,
    pub dims: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct TupleCtorInstance {
    pub dims: Vec<Value>,
}

// Environment
#[derive(Debug, Clone)]
pub struct Env {
    pub globals: HashMap<String, Value>,
    pub locals: HashMap<String, Value>,
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
