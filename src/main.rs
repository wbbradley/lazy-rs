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
    pub(crate) mod exec;
}

use clap::Parser;
use test_each_file::test_each_path;

use crate::{
    env::Env,
    error::{error, PitaError},
    id::{value_from_id, IdImpl},
    runtime::{
        error::RuntimeError,
        exec::{walk, Continuation, ContinuationChoice},
    },
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
    for d in dbg!(decls) {
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
    let result = walk_tree(env, entrypoint);
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err(PitaError::from(e)),
    }
}

fn walk_tree(env: Env, mut expr: Value) -> Result<Value, RuntimeError> {
    // Return a value in WHNF.
    // State is maintained in the expr register and the continuation list.
    let message = format!("Walk({expr:?})");
    let mut continuation = walk(env, message);

    loop {
        tracing::debug!("walk_tree loop on {continuation:?}");
        continuation = match continuation.choice {
            ContinuationChoice::Done => {
                if let Some(next) = continuation.next {
                    // Push this value on to the next continuation.
                    let (new_expr, new_continuation) = (*next).prepare(expr)?;
                    expr = new_expr;
                    new_continuation
                } else {
                    break Ok(expr);
                }
            }
            ContinuationChoice::Walk { .. } if expr.is_weak_head_normal_form() => Continuation {
                message: "from a Walk".to_string(),
                choice: ContinuationChoice::Done,
                next: continuation.next,
            },
            ContinuationChoice::Walk { ref env } => match expr {
                Value::Id(id) => {
                    let new_expr = dbg!(env)
                        .get_symbol(&id)
                        .ok_or(RuntimeError::UnresolvedSymbol(id))?
                        .clone();
                    expr = new_expr;
                    continue;
                }
                Value::Callsite { function, argument } => {
                    // Evaluate the callee, then apply the arguments to it.
                    expr = *function;
                    // Chain the continuation.
                    continuation.next = Some(Box::new(Continuation {
                        message: "callsite walk".to_string(),
                        choice: ContinuationChoice::Application {
                            env: env.clone(),
                            argument: *argument,
                        },
                        next: continuation.next,
                    }));
                    // Re-use the existing continuation for perf.
                    continue;
                }
                Value::Match {
                    subject: _,
                    pattern_exprs: _,
                } => todo!(),
                Value::Tuple { dims: _ } => todo!(),
                Value::Thunk { env: _, expr: _ } => todo!(),
                Value::Builtin(_f) => todo!(),
                Value::Let {
                    name: _,
                    value: _,
                    body: _,
                } => todo!(),
                _ => todo!("handle {expr:?} in Walk"),
            },
            ContinuationChoice::Match { .. } => todo!(),
            ContinuationChoice::Application { .. } => todo!(),
            ContinuationChoice::Thunk { .. } => todo!(),
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
