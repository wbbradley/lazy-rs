use std::{collections::HashMap, rc::Rc};

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
pub struct Env {
    globals: HashMap<String, Rc<Value>>,
    locals: HashMap<String, Rc<Value>>,
}

#[derive(Clone)]
pub enum Value {
    Lambda(Rc<Lambda>),
    Literal(LiteralValue),
    Ctor(Rc<Ctor>),
    CtorInstance(Rc<CtorInstance>),
    TupleCtorInstance(Rc<TupleCtorInstance>),
    Thunk(Rc<RefCell<Thunk>>),
    Builtin(Rc<dyn Fn(Vec<Value>) -> Result<Value>>),
}

#[derive(Clone)]
pub struct Thunk {
    env: Env,
    expr: Box<Expr>,
    memoized: Option<Rc<Value>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: HashMap::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::new();
        env.add_builtin("+", |args| {
            if let [Value::Literal(LiteralValue::Int(a)), Value::Literal(LiteralValue::Int(b))] =
                &args[..]
            {
                Ok(Value::Literal(LiteralValue::Int(a + b)))
            } else {
                Err(RuntimeError::InvalidCallsite(
                    "+ requires two integers".into(),
                ))
            }
        });
        // Add other builtins...
        env
    }

    pub fn add_builtin<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Vec<Value>) -> Result<Value> + 'static,
    {
        self.globals
            .insert(name.to_string(), Rc::new(Value::Builtin(Rc::new(f))));
    }

    pub fn get_symbol(&self, name: &str) -> Option<Rc<Value>> {
        self.locals
            .get(name)
            .cloned()
            .or_else(|| self.globals.get(name).cloned())
    }

    pub fn add_symbol(&mut self, name: String, value: Rc<Value>) {
        self.locals.insert(name, value);
    }
}

pub fn eval(env: &Env, expr: &Expr) -> Result<Rc<Value>> {
    match expr {
        Expr::Literal(lit) => Ok(Rc::new(Value::Literal(lit.clone()))),
        Expr::Id(id) => env
            .get_symbol(&id.name)
            .ok_or_else(|| RuntimeError::UnresolvedSymbol(id.name.clone())),
        Expr::Lambda(lambda) => Ok(Rc::new(Value::Lambda(Rc::new(lambda.clone())))),
        Expr::Ctor(ctor) => Ok(Rc::new(Value::Ctor(Rc::new(ctor.clone())))),
        Expr::Let(let_expr) => {
            let mut new_env = env.clone();
            let value = Rc::new(Value::Thunk(Rc::new(RefCell::new(Thunk {
                env: env.clone(),
                expr: Box::new(let_expr.value.as_ref().clone()),
                memoized: None,
            }))));
            new_env.add_symbol(let_expr.name.name.clone(), value);
            eval(&new_env, &let_expr.body)
        }
        Expr::Match(match_expr) => eval_match(env, match_expr),
        Expr::Callsite(callsite) => eval_callsite(env, callsite),
        Expr::TupleCtor(tuple) => {
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
