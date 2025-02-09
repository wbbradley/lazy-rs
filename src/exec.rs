#![allow(dead_code, unused_variables)]
use std::{cell::RefCell, rc::Rc};

use crate::{
    env::Env,
    value::{PatternExpr, Predicate, Value},
};
type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    UnresolvedSymbol(String),
    InvalidDecl(String),
    InvalidCallsite(String),
    NoMatch(String),
    MatchTypeError(String),
}

#[derive(Debug)]
pub enum Step {
    Done(Value),
    Continuation(Continuation),
}

fn is_weak_head_normal_form(value: &Value) -> bool {
    matches!(
        value,
        Value::Int(_) | Value::Str(_) | Value::Lambda { .. } | Value::Ctor { .. }
    )
}

/*
pub fn advance() -> Result<Step> {
    match self {
        Step::Done(value) => todo!(),
        Step::Continuation(continuation) => todo!(),
        Step::Done(value) => {
            assert!(is_weak_head_normal_form(&value));
            panic!()
        }
        Step::Walk { env, expr, next } => match expr {
            Value::Int(x) => Ok(Step::Done(Value::Int(x))),
            Value::Str(x) => Ok(Step::Done(Value::Str(x))),
            Value::Id(id) => {
                if let Some(value) = env.get_symbol(&id) {
                    Ok(Step::Walk {
                        env,
                        expr: value.clone(),
                        next,
                    })
                } else {
                    Err(RuntimeError::UnresolvedSymbol(id.name().to_string()))
                }
            }
            Value::Let { name, value, body } => {
                // NB: the next step of the computation does not alter the environment, because
                // the updated environment is captured in the thunk.
                Ok(Step::Done(Value::Thunk {
                    env: env.add_symbol(name, *value),
                    expr: Rc::new(RefCell::new(*body)),
                }))
            }
            Value::Match(match_expr) => Ok(Step::Walk {
                env: env.clone(),
                expr: match_expr.subject,
                next: Some(Rc::new(RefCell::new(ContinuationChoice::Match {
                    env,
                    patterns: match_expr.pattern_exprs,
                    next,
                }))),
            }),
            Value::Callsite(callsite) => Ok(Step::Walk {
                env: env.clone(),
                expr: callsite.function,
                next: Some(Rc::new(RefCell::new(Continuation::Callsite {
                    env,
                    arguments: callsite.arguments,
                    next,
                }))),
            }),
            Value::TupleCtor(tuple) => {
                let dims = tuple
                    .dims
                    .into_iter()
                    .map(|expr| {
                        Rc::new(Value::Thunk(Rc::new(RefCell::new(Thunk {
                            env: env.clone(),
                            expr: Box::new(expr),
                            memoized: None,
                        }))))
                    })
                    .collect();
                Ok(Step::Done(Rc::new(Value::Tuple(Rc::new(
                    TupleCtorInstance { dims },
                )))))
            }
            Value::Lambda(lambda) => Ok(Step::Done(Rc::new(Value::Lambda(Rc::new(lambda))))),
            Value::Ctor(ctor) => Ok(Step::Done(Rc::new(Value::Ctor(Rc::new(ctor))))),
        },
    }
}
        */

#[derive(Debug)]
pub struct Continuation {
    pub env: Env,
    pub message: String,
    pub choice: ContinuationChoice,
    pub next: Option<RefCell<Box<Continuation>>>,
}

#[derive(Debug)]
pub enum ContinuationChoice {
    Done {
        value: Value,
    },
    Walk {
        expr: Value,
    },
    Match {
        subject: Value,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        function: Value,
        arguments: Vec<Value>,
    },
    Thunk {
        env: Env,
        // Envs that share the same thunks will share the memoized value.
        expr: Rc<RefCell<Value>>,
    },
}

impl Continuation {
    pub fn walk(env: Env, expr: Value, message: String) -> Self {
        Continuation {
            env,
            message,
            choice: ContinuationChoice::Walk { expr },
            next: None,
        }
    }
    pub fn prepare(self, value: Value) -> Result<Self> {
        match self.choice {
            ContinuationChoice::Done { value } => {
                panic!("why are we preparing when we're Done? {value:#?}")
            }
            ContinuationChoice::Walk { expr } => {
                panic!("shouldn't have to prepare a Walk {expr:#?}")
            }
            ContinuationChoice::Match {
                subject,
                pattern_exprs,
            } => todo!(),
            ContinuationChoice::Callsite {
                function,
                arguments,
            } => todo!(),
            ContinuationChoice::Thunk { env, expr } => todo!(),
        }
    }
}

fn match_pattern(predicate: &Predicate, value: &Value, env: &Env) -> Option<Env> {
    todo!()
}
