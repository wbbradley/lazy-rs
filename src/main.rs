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

use clap::Parser;
use test_each_file::test_each_path;

use crate::{
    env::Env,
    error::{error, PitaError},
    id::{value_from_id, IdImpl},
    runtime::error::RuntimeError,
    value::Value,
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
    let mut env = Env::with_builtins();
    for d in decls {
        tracing::info!("{:#?}", d);
        env.add_symbol_mut(d.name, d.body);
    }
    // Build an entrypoint which is a call to user `main`.
    let entrypoint = Value::Callsite {
        function: Box::new(value_from_id::<IdImpl>("main")),
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
                        state = State::Walk {
                            env,
                            expr: env
                                .get_symbol(&id)
                                .ok_or(RuntimeError::UnresolvedSymbol(id))?
                                .clone(),
                        };
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
                    Continuation::ApplyTo { env, arg, next } => {
                        tracing::info!("applying {expr:?} to {arg:?}");
                        let Value::Lambda { param, body } = expr else {
                            panic!("expected lambda, got {expr:?}");
                        };
                        // Apply the arguments to the function.
                        env.add_symbol(param, arg);
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
