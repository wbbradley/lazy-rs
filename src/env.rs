use std::collections::HashMap;

use crate::value::Value;

#[derive(Debug, Clone)]
pub(crate) struct Env {
    bindings: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
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

    pub fn get_symbol(&self, symbol: &str) -> Option<Value> {
        self.bindings.get(symbol).cloned()
    }

    pub fn add_global_symbol(&mut self, symbol: String, value: Value) {
        self.bindings.insert(symbol, value);
    }

    pub fn add_symbol(&mut self, symbol: String, value: Value) {
        self.bindings.insert(symbol, value);
    }
    pub fn add_builtin<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Vec<Value>) -> Value + 'static,
    {
        self.bindings
            .insert(name.to_string(), Value::builtin(std::rc::Rc::new(f)));
    }
}
