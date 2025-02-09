#![allow(dead_code, unused_variables, unused_imports)]
use std::{cell::RefCell, rc::Rc};

use crate::{
    env::Env,
    runtime::error::RuntimeError,
    value::{PatternExpr, Predicate, Value},
};

#[derive(Debug)]
pub enum Step {
    Done(Value),
    Continuation(Continuation),
}

pub fn is_weak_head_normal_form(value: &Value) -> bool {
    matches!(
        value,
        Value::Int(_)
            | Value::Str(_)
            | Value::Lambda { .. }
            | Value::Ctor { .. }
            | Value::Builtin { .. }
            | Value::Tuple { .. }
    )
}

#[derive(Debug)]
pub struct Continuation {
    pub message: String,
    pub choice: ContinuationChoice,
    pub next: Option<Box<Continuation>>,
}

#[derive(Debug)]
pub enum ContinuationChoice {
    Done,
    Walk {
        env: Env,
    },
    Match {
        env: Env,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        env: Env,
        arguments: Vec<Value>,
    },
    Thunk {
        env: Env,
        // Envs that share the same thunks will share the memoized value.
        expr: Rc<RefCell<Value>>,
    },
}

impl Continuation {
    pub fn walk(env: Env, message: String) -> Self {
        Continuation {
            message,
            choice: ContinuationChoice::Walk { env },
            next: None,
        }
    }
    pub fn prepare(self, value: Value) -> Result<(Value, Self), RuntimeError> {
        match self.choice {
            ContinuationChoice::Done => {
                panic!("why are we preparing when we're Done? {value:#?}")
            }
            ContinuationChoice::Walk { env: _ } => {
                panic!("shouldn't have to prepare a Walk {value:#?}")
            }
            ContinuationChoice::Match {
                env: _,
                pattern_exprs: _,
            } => {
                // subject = value
                todo!();
            }
            ContinuationChoice::Callsite {
                env: _,
                arguments: _,
            } => {
                // function = value;
                todo!();
            }
            ContinuationChoice::Thunk { env: _, expr: _ } => todo!(),
        }
    }
}

fn match_pattern(predicate: &Predicate, value: &Value, env: &Env) -> Option<Env> {
    todo!()
}
