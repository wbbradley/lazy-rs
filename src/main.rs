//! Pita is a programming language for writing lazy functional programs.
mod env;
mod error;
mod id;
mod location;
mod parser;
mod token;
mod value;

mod runtime {
    pub(crate) mod error;
}

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use clap::Parser;
use test_each_file::test_each_path;
use value::Predicate;

use crate::{
    env::Env,
    error::{error, PitaError},
    id::{gensym, internal_id, value_from_id, Id, IdImpl},
    runtime::error::RuntimeError,
    value::{Decl, PatternExpr, Value},
};

#[derive(Parser)]
struct Args {
    /// The file to execute
    filename: String,
}

fn main() -> Result<(), PitaError> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let value = run_program(args.filename)?;
    tracing::info!("{:#?}", value);
    Ok(())
}

struct DefBuilder {
    name: Id,
    arity: usize,
    variant: DefBuilderVariant,
}

enum DefBuilderVariant {
    // f = 3.14
    Value(Value),
    // f x 3 = 9001
    // f x y = (g x)
    Patterns(Vec<PatternExpr>),
}

fn merge_decl(all_symbols: &mut HashMap<String, DefBuilder>, decl: Decl) -> Result<(), PitaError> {
    let name = decl.name.to_string();
    if let Some(def_builder) = all_symbols.get_mut(&name) {
        if def_builder.arity != decl.patterns.len() {
            return Err(error!(
                "a decl for {name} does not match another in terms of arguments count \
                    [{}, {}]",
                def_builder.arity,
                decl.patterns.len()
            ));
        }
    } else {
        all_symbols.insert(
            name,
            DefBuilder {
                name: decl.name,
                arity: decl.patterns.len(),
                variant: if decl.patterns.is_empty() {
                    DefBuilderVariant::Value(decl.body)
                } else {
                    DefBuilderVariant::Patterns(vec![PatternExpr {
                        predicate: Predicate::Tuple(decl.patterns),
                        expr: decl.body,
                    }])
                },
            },
        );
    }
    Ok(())
}

fn build_symbol(def_builder: DefBuilder) -> Result<(Id, Value), PitaError> {
    match def_builder.variant {
        DefBuilderVariant::Patterns(pattern_exprs) => {
            assert!(!pattern_exprs.is_empty());
            // Create callsite bindings.
            let param_names: Vec<Id> = (0..def_builder.arity)
                .map(|_| gensym(def_builder.name.location()))
                .collect();
            // Building this:
            // f = \x.\y.\z. match (x, y, z) {
            //   <pattern_exprs...>
            // }
            let inner_body = Value::Match {
                // Match multi-parameter function arguments with tuples.
                subject: Box::new(Value::Tuple {
                    dims: param_names
                        .iter()
                        .map(value_from_id::<IdImpl>)
                        .collect::<Vec<Value>>(),
                }),
                pattern_exprs,
            };

            // We have a predicate to match.
            Ok((
                def_builder.name,
                param_names
                    .into_iter()
                    .rev()
                    .fold(inner_body, |value, acc| Value::Lambda {
                        param: acc,
                        body: Box::new(value),
                    }),
            ))
        }
        DefBuilderVariant::Value(value) => Ok((
            def_builder.name,
            // Thunk the value since it is a singleton.
            Value::Thunk {
                // Env to be supplied by the runtime.
                // TODO: mark this as viewing the global env somehow.
                env: None,
                expr: Rc::new(RefCell::new(value)),
            },
        )),
    }
}

fn build_env(all_symbols: &mut HashMap<String, DefBuilder>) -> Result<Env, PitaError> {
    let mut env = Env::with_builtins();
    for (_, def_builder) in all_symbols.drain() {
        // This loop handles defining a single global variable as a function or otherwise.
        let (name, value) = build_symbol(def_builder)?;
        env.add_symbol_mut(name, value);
    }
    Ok(env)
}

fn run_program(filename: impl AsRef<std::path::Path>) -> Result<Value, PitaError> {
    let filename = filename.as_ref();
    if !filename.exists() {
        return Err(error!("file {filename:?} does not exist"));
    }
    let content = std::fs::read_to_string(filename)?;
    let filename = filename.display().to_string().leak();
    let file_span = crate::parser::Span::new_extra(&content, filename);
    let (remaining, decls) = parser::program_parser(file_span)?;
    if remaining.len() != 0 {
        return Err(error!("remaining input: {remaining:?}"));
    }
    let mut all_symbols = Default::default();
    for decl in decls {
        merge_decl(&mut all_symbols, decl)?;
    }
    let env = build_env(&mut all_symbols)?;

    // Build an entrypoint which is a call to user `main`.
    let entrypoint = Value::Callsite {
        function: Box::new(value_from_id::<IdImpl>(&internal_id("main"))),
        argument: Box::new(Value::Tuple {
            dims: vec![
                // TODO: include command-line arguments.
            ],
        }),
    };
    let result = eval_loop(env, entrypoint);
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err(PitaError::from(e)),
    }
}

fn eval_loop(env: Env, expr: Value) -> Result<Value, RuntimeError> {
    enum Continuation {
        ApplyTo {
            env: Env,
            arg: Value,
            next: Box<Continuation>,
        },
        Done,
    }
    enum State {
        Walk { env: Env, expr: Value },
        ContinueWith(Value),
    }
    let global_env = env.clone();
    let mut state: State = State::Walk { env, expr };
    let mut continuation = Continuation::Done;

    loop {
        match state {
            State::Walk { env, expr } => {
                // The job of Walk is to ensure that the expression is in WHNF.
                if expr.is_weak_head_normal_form() {
                    state = State::ContinueWith(expr);
                    continue;
                }
                match expr {
                    Value::Id(id) => {
                        let expr = env
                            .get_symbol(&id)
                            .ok_or(RuntimeError::UnresolvedSymbol(id))?
                            .clone();
                        state = State::Walk { env, expr };
                    }
                    Value::Callsite { function, argument } => {
                        // Evaluate the callee, then apply the arguments to it.
                        state = State::Walk {
                            env,
                            expr: *function,
                        };
                        // Chain the continuation.
                        continuation = Continuation::ApplyTo {
                            env: global_env.clone(),
                            arg: *argument,
                            next: Box::new(continuation),
                        };
                    }
                    _ => todo!("handle {expr:?} in Walk"),
                }
            }
            State::ContinueWith(expr) => {
                match continuation {
                    Continuation::ApplyTo { mut env, arg, next } => {
                        tracing::info!("applying {expr:?} to {arg:?}");
                        let Value::Lambda { param, body } = expr else {
                            panic!("expected lambda, got {expr:?}");
                        };
                        // Apply the arguments to the function.
                        env.add_symbol_mut(param, arg);
                        state = State::Walk { env, expr: *body };
                        continuation = *next;
                    }
                    Continuation::Done => {
                        return Ok(expr);
                    }
                }
            }
        }
    }
}

test_each_path! { in "./tests" => test::test_pita_file }

#[cfg(test)]
mod test {
    use crate::run_program;

    pub(crate) fn test_pita_file(filename: &std::path::Path) {
        let result = run_program(filename);
        assert!(result.is_ok(), "running {filename:?}: {result:?}");
    }
}
