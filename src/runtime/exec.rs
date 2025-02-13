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
    Application {
        env: Env,
        argument: Value,
    },
    Thunk {
        env: Env,
        // Envs that share the same thunks will share the memoized value.
        expr: Rc<RefCell<Value>>,
    },
}

pub fn walk(env: Env, message: String) -> Continuation {
    Continuation {
        message,
        choice: ContinuationChoice::Walk { env },
        next: None,
    }
}
impl Continuation {
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
            ContinuationChoice::Application {
                env: _,
                argument: _,
            } => {
                todo!();
                /*
                // function = value;
                match &value {
                    Value::Ctor { name, dims } => {
                        // Apply the arguments to the function.
                        tracing::info!("applying {name:?} to {dims:?}");
                        Ok((
                            //Value::Ctor{ name:
                            Continuation {
                                message: "ctor is in whnf".to_string(),
                                choice: ContinuationChoice::Done,
                                next: self.next,
                            },
                        ))
                    }
                    _ => todo!(),
                }
                */
            }
            ContinuationChoice::Thunk { env: _, expr: _ } => todo!(),
        }
    }
}

fn match_pattern(predicate: &Predicate, value: &Value, env: &Env) -> Option<Env> {
    todo!()
}
