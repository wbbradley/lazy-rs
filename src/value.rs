#![allow(dead_code)]
use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{env::Env, parser::KEYWORDS};

#[derive(Debug, Clone)]
pub enum Predicate {
    Id(Id),
    Int(i64),
    Tuple(Vec<Predicate>),
    Ctor(Id, Vec<Predicate>),
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub name: Id,
    pub pattern: Vec<Predicate>,
    pub body: Value,
}

#[derive(Debug, Clone)]
pub struct PatternExpr {
    pub predicate: Predicate,
    pub expr: Value,
}

#[derive(Debug, Clone)]
pub struct Id {
    name: String,
}

#[derive(Debug, Clone)]
pub struct IdError(pub String);

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Id must start with an alphabetic letter or valid punctuation: '{}'",
            self.0
        )
    }
}
impl std::error::Error for IdError {}

impl std::str::FromStr for Id {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::is_valid(s) {
            Err(IdError(s.to_string()))
        } else {
            Ok(Self {
                name: s.to_string(),
            })
        }
    }
}

impl From<&'static str> for Id {
    fn from(s: &'static str) -> Self {
        debug_assert!(Self::is_valid(s));
        Self {
            name: s.to_string(),
        }
    }
}

impl Id {
    pub fn is_valid(id: &str) -> bool {
        id.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c.is_ascii_punctuation())
            && !KEYWORDS.iter().any(|&kwd| kwd == id)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct CtorId {
    name: String,
}

#[derive(Debug, Clone)]
pub struct CtorIdError(pub String);

impl std::fmt::Display for CtorIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CtorId must start with an alphabetic letter or valid punctuation: '{}'",
            self.0
        )
    }
}
impl std::error::Error for CtorIdError {}

impl std::str::FromStr for CtorId {
    type Err = CtorIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::is_valid(s) {
            Err(CtorIdError(s.to_string()))
        } else {
            Ok(Self {
                name: s.to_string(),
            })
        }
    }
}

impl From<&'static str> for CtorId {
    fn from(s: &'static str) -> Self {
        debug_assert!(Self::is_valid(s));
        Self {
            name: s.to_string(),
        }
    }
}

impl CtorId {
    pub fn is_valid(ctor_id: &str) -> bool {
        ctor_id.chars().next().is_some_and(|c| !c.is_uppercase())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub type Builtin = dyn Fn(Vec<Value>) -> std::result::Result<Value, crate::runtime::RuntimeError>;

// Runtime values
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Str(String),
    Null,
    Lambda {
        param_names: Vec<Id>,
        body: Box<Value>,
    },
    Id(Id),
    Match {
        subject: Box<Value>,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        function: Box<Value>,
        arguments: Vec<Value>,
    },
    Tuple {
        dims: Vec<Value>,
    },
    Thunk {
        env: Env,
        // Envs that share the same thunks will share the memoized value.
        expr: Rc<RefCell<Value>>,
    },
    Builtin(Rc<Builtin>),
    Let {
        name: Id,
        value: Box<Value>,
        body: Box<Value>,
    },
    Ctor {
        name: CtorId,
        dims: Vec<Value>,
    },
}

impl Value {
    pub(crate) fn builtin(f: Rc<Builtin>) -> Self {
        Self::Builtin(f)
    }
    /*pub(crate) fn id(name: impl AsRef<str>) -> Self {
        Self::Id(Id::new(name))
    }*/
}
impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(_) => todo!(),
            Value::Str(_) => todo!(),
            Value::Null => todo!(),
            Value::Lambda { param_names, body } => todo!(),
            Value::Id(id) => f.write_str(id.name()),
            Value::Match {
                subject,
                pattern_exprs,
            } => todo!(),
            Value::Callsite {
                function,
                arguments,
            } => write!(f, "({:?} {:?})", function, arguments),
            Value::Tuple { dims } => todo!(),
            Value::Thunk { env, expr } => todo!(),
            Value::Builtin(rc) => todo!(),
            Value::Let { name, value, body } => todo!(),
            Value::Ctor { name, dims } => todo!(),
        }
    }
}
