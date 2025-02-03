use std::{cell::RefCell, rc::Rc};

use crate::{env::Env, value::Value};

type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    UnresolvedSymbol(String),
    InvalidDecl(String),
    InvalidCallsite(String),
    NoMatch(String),
    MatchTypeError(String),
}

pub fn eval(env: &Env, expr: Value) -> Result<Rc<Value>> {
    match expr {
        Value::Literal(lit) => Ok(Rc::new(Value::Literal(lit.clone()))),
        Value::Id(id) => env
            .get_symbol(&id.name)
            .ok_or_else(|| RuntimeError::UnresolvedSymbol(id.name.clone())),
        Value::Lambda(lambda) => Ok(Rc::new(Value::Lambda(Rc::new(lambda.clone())))),
        Value::Ctor(ctor) => Ok(Rc::new(Value::Ctor(Rc::new(ctor.clone())))),
        Value::Let(let_expr) => {
            let mut new_env = env.clone();
            let value = Rc::new(Value::Thunk(Rc::new(RefCell::new(Thunk {
                env: env.clone(),
                expr: Box::new(let_expr.value.as_ref().clone()),
                memoized: None,
            }))));
            new_env.add_symbol(let_expr.name.name.clone(), value);
            eval(&new_env, &let_expr.body)
        }
        Value::Match(match_expr) => eval_match(env, match_expr),
        Value::Callsite(callsite) => eval_callsite(env, callsite),
        Value::TupleCtor(tuple) => {
            let dims = tuple
                .dims
                .iter()
                .map(|expr| {
                    Ok(Value::Thunk(Rc::new(RefCell::new(Thunk {
                        env: env.clone(),
                        expr: Box::new(expr.clone()),
                        memoized: None,
                    }))))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Rc::new(Value::TupleCtorInstance(Rc::new(
                TupleCtorInstance {
                    dims: dims.into_iter().map(Rc::new).collect(),
                },
            ))))
        }
    }
}
