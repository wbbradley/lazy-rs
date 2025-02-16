#![allow(dead_code)]
use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{env::Env, id::Id, location::Location, token::Token};

#[derive(Debug, Clone)]
pub enum Predicate {
    Irrefutable(Id),
    Int(i64, Location),
    Tuple(Vec<Predicate>),
    Ctor(Id, Vec<Predicate>),
}

impl Predicate {
    pub fn location(&self) -> Location {
        match self {
            Predicate::Irrefutable(id) => id.location(),
            Predicate::Int(_, loc) => *loc,
            Predicate::Tuple(predicates) => predicates[0].location(),
            Predicate::Ctor(_, predicates) => predicates[0].location(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub name: Id,
    pub patterns: Vec<Predicate>,
    pub body: Value,
}

#[derive(Debug, Clone)]
pub struct PatternExpr {
    pub predicate: Predicate,
    pub expr: Value,
}

#[derive(Debug, Clone)]
pub struct CtorId {
    name: String,
}

#[derive(Debug, Clone)]
pub struct CtorIdError(pub Token);

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

pub type Builtin =
    dyn Fn(Vec<Value>) -> std::result::Result<Value, crate::runtime::error::RuntimeError>;

// Runtime values
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Str(String),
    Null,
    Lambda {
        param: Id,
        body: Box<Value>,
    },
    Id(Id),
    Match {
        subject: Box<Value>,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        function: Box<Value>,
        argument: Box<Value>,
    },
    Tuple {
        dims: Vec<Value>,
    },
    Thunk {
        env: Option<Env>,
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
    pub fn is_weak_head_normal_form(&self) -> bool {
        matches!(
            self,
            Value::Int(_)
                | Value::Str(_)
                | Value::Lambda { .. }
                | Value::Ctor { .. }
                | Value::Builtin { .. }
                | Value::Tuple { .. }
        )
    }
}
impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(x) => write!(f, "{x}"),
            Value::Str(x) => {
                f.write_str("\"")?;
                for c in x.chars() {
                    if c.is_ascii_control() {
                        write!(f, "\\x{:02x}", c as u8)?;
                    } else if c == '"' {
                        f.write_str("\\\"")?;
                    } else {
                        write!(f, "{c}")?;
                    }
                }
                f.write_str("\"")
            }
            Value::Null => todo!(),
            Value::Lambda { param, body } => write!(f, "λ{param}.{body:?}"),
            Value::Id(id) => f.write_str(id.name()),
            Value::Match {
                subject,
                pattern_exprs,
            } => {
                write!(f, "match {subject:?} {{ {pattern_exprs:?} }}",)
            }
            Value::Callsite { function, argument } => write!(f, "({:?} {:?})", function, argument),
            Value::Tuple { dims } => {
                f.write_str("(")?;
                let mut delim = "";
                for dim in dims {
                    write!(f, "{delim}{:?}", dim)?;
                    delim = " ";
                }
                f.write_str(")")
            }
            Value::Thunk { .. } => todo!(),
            Value::Builtin(_) => "<builtin>".fmt(f),
            Value::Let { .. } => todo!(),
            Value::Ctor { .. } => todo!(),
        }
    }
}
