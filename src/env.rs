use crate::runtime::RuntimeError;
use std::rc::Rc;

use crate::value::{Id, Value};

#[derive(Debug, Clone)]
pub(crate) struct Env {
    bindings: rpds::RedBlackTreeMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            bindings: Default::default(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::new();
        env.add_builtin("+", |args| {
            if let [Value::Int(a), Value::Int(b)] = &args[..] {
                Ok(Value::Int(a + b))
            } else {
                Err(RuntimeError::InvalidCallsite(
                    "+ requires two integers".into(),
                ))
            }
        });
        // Add other builtins...
        env
    }
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.bindings.contains_key(symbol)
    }

    pub fn get_symbol(&self, symbol: &Id) -> Option<&Value> {
        self.bindings.get(symbol.name())
    }

    pub fn add_global_symbol(&mut self, symbol: &Id, value: Value) {
        self.bindings.insert_mut(symbol.name().to_string(), value);
    }

    pub fn add_symbol(&mut self, symbol: &Id, value: Value) {
        self.bindings.insert_mut(symbol.name().to_string(), value);
    }
    pub fn add_builtin<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Vec<Value>) -> Result<Value, RuntimeError> + 'static,
    {
        self.bindings
            .insert(name.to_string(), Value::builtin(Rc::new(f)));
    }
}
