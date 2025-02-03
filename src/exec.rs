use std::{cell::RefCell, rc::Rc};

use crate::{
    data::{Expr, Value},
    runtime::Env,
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

#[derive(Clone)]
pub struct Continuation {
    env: Env,
    next: Option<Rc<RefCell<Continuation>>>,
}

#[derive(Clone)]
pub enum Step {
    Done(Rc<Value>),
    Walk {
        env: Env,
        expr: Box<Expr>,
        next: Option<Rc<RefCell<Continuation>>>,
    },
}

impl Step {
    fn advance(self) -> Result<Step> {
        match self {
            Step::Done(value) => Ok(Step::Done(value)),
            Step::Walk { env, expr, next } => match *expr {
                Expr::Literal(lit) => Ok(Step::Done(Rc::new(Value::Literal(lit)))),
                Expr::Id(ref id) => {
                    if let Some(value) = env.get_symbol(&id.name) {
                        Ok(Step::Walk {
                            env,
                            expr: Box::new(value.into()),
                            next,
                        })
                    } else {
                        Err(RuntimeError::UnresolvedSymbol(id.name.clone()))
                    }
                }
                Expr::Let(let_expr) => {
                    let mut new_env = env.clone();
                    let thunk = Rc::new(Value::Thunk(Rc::new(RefCell::new(Thunk {
                        env: env.clone(),
                        expr: let_expr.value,
                        memoized: None,
                    }))));
                    new_env.add_symbol(let_expr.name.name.clone(), thunk);
                    Ok(Step::Walk {
                        env: new_env,
                        expr: let_expr.body,
                        next,
                    })
                }
                Expr::Match(match_expr) => Ok(Step::Walk {
                    env: env.clone(),
                    expr: match_expr.subject,
                    next: Some(Rc::new(RefCell::new(Continuation::Match {
                        env,
                        patterns: match_expr.pattern_exprs,
                        next,
                    }))),
                }),
                Expr::Callsite(callsite) => Ok(Step::Walk {
                    env: env.clone(),
                    expr: callsite.function,
                    next: Some(Rc::new(RefCell::new(Continuation::Callsite {
                        env,
                        arguments: callsite.arguments,
                        next,
                    }))),
                }),
                Expr::TupleCtor(tuple) => {
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
                    Ok(Step::Done(Rc::new(Value::TupleCtorInstance(Rc::new(
                        TupleCtorInstance { dims },
                    )))))
                }
                Expr::Lambda(lambda) => Ok(Step::Done(Rc::new(Value::Lambda(Rc::new(lambda))))),
                Expr::Ctor(ctor) => Ok(Step::Done(Rc::new(Value::Ctor(Rc::new(ctor))))),
            },
        }
    }
}

#[derive(Clone)]
enum Continuation {
    Match {
        env: Env,
        patterns: Vec<PatternExpr>,
        next: Option<Rc<RefCell<Continuation>>>,
    },
    Callsite {
        env: Env,
        arguments: Vec<Expr>,
        next: Option<Rc<RefCell<Continuation>>>,
    },
    Thunk {
        thunk: Rc<RefCell<Thunk>>,
        next: Option<Rc<RefCell<Continuation>>>,
    },
}

impl Continuation {
    fn prepare(self, value: Rc<Value>) -> Result<Step> {
        match self {
            Continuation::Match {
                env,
                patterns,
                next,
            } => {
                for pattern in patterns {
                    match match_pattern(&pattern.predicate, &value, &env)? {
                        Some(new_env) => {
                            return Ok(Step::Walk {
                                env: new_env,
                                expr: Box::new(pattern.expr),
                                next,
                            });
                        }
                        None => continue,
                    }
                }
                Err(RuntimeError::NoMatch("No pattern matched".into()))
            }
            Continuation::Callsite {
                env,
                arguments,
                next,
            } => match &*alue {
                Value::Lambda(lambda) => {
                    if arguments.len() != lambda.param_names.len() {
                        return Err(RuntimeError::InvalidCallsite(
                            "Wrong number of arguments".into(),
                        ));
                    }
                    let mut new_env = env.clone_globals();
                    for (param, arg) in lambda.param_names.iter().zip(arguments) {
                        new_env.add_symbol(
                            param.name.clone(),
                            Rc::new(Value::Thunk(Rc::new(RefCell::new(Thunk {
                                env: env.clone(),
                                expr: Box::new(arg),
                                memoized: None,
                            })))),
                        );
                    }
                    Ok(Step::Walk {
                        env: new_env,
                        expr: lambda.body.clone(),
                        next,
                    })
                }
                Value::Ctor(ctor) => {
                    let dims = arguments
                        .into_iter()
                        .map(|arg| {
                            Rc::new(Value::Thunk(Rc::new(RefCell::new(Thunk {
                                env: env.clone(),
                                expr: Box::new(arg),
                                memoized: None,
                            }))))
                        })
                        .collect();
                    Ok(Step::Done(Rc::new(Value::CtorInstance(Rc::new(
                        CtorInstance {
                            ctor: ctor.clone(),
                            dims,
                        },
                    )))))
                }
                Value::Builtin(f) => {
                    let mut evaluated_args = Vec::new();
                    for arg in arguments {
                        let arg_value = eval(&env, &arg)?;
                        evaluated_args.push(arg_value);
                    }
                    Ok(Step::Done(Rc::new(f(evaluated_args)?)))
                }
                _ => Err(RuntimeError::InvalidCallsite(
                    "Called non-function value".into(),
                )),
            },
            Continuation::Thunk { thunk, next } => {
                thunk.borrow_mut().memoized = Some(value.clone());
                match next {
                    Some(next) => next.borrow().clone().prepare(value),
                    None => Ok(Step::Done(value)),
                }
            }
        }
    }
}

pub fn eval(env: &Env, expr: &Expr) -> Result<Rc<Value>> {
    let mut step = Step::Walk {
        env: env.clone(),
        expr: Box::new(expr.clone()),
        next: None,
    };

    loop {
        match step {
            Step::Done(value) => return Ok(value),
            _ => step = step.advance()?,
        }
    }
}
