//! Pita is a programming language for writing lazy functional programs.
use crate::error::PitaError;
use test_each_file::test_each_path;

mod env;
mod error;
mod location;
mod parser;
mod token;
mod value;

mod runtime {
    pub(crate) mod error;
    pub(crate) mod exec;
}

// Example clap arguments.
use crate::{
    env::Env,
    runtime::error::RuntimeError,
    runtime::exec::{is_weak_head_normal_form, Continuation, ContinuationChoice},
    value::Value,
};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The file to execute
    file: String,
}

fn main() -> Result<(), PitaError> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let value = run_program(args)?;
    tracing::info!("{:#?}", value);
    Ok(())
}

fn run_program(filename: impl AsRef<std::path::Path>) -> Result<Value, PitaError> {
    let input = std::fs::read_to_string(filename).expect("reading file");
    let filename = filename.as_ref().display().to_string();
    let decls = parser::program_parser(filename, &input)?;
    let mut env = Env::with_builtins();
    for d in decls.1 {
        tracing::info!("{:#?}", d);
        env.add_symbol_mut(d.name, d.body);
    }
    // Build an entrypoint which is a call to user `main`.
    let entrypoint = Value::Callsite {
        function: Box::new(Value::Id("main".into())),
        arguments: vec![],
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
    let mut continuation: Continuation = Continuation::walk(env, message);
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
            ContinuationChoice::Walk { .. } if is_weak_head_normal_form(&expr) => Continuation {
                message: "from a Walk".to_string(),
                choice: ContinuationChoice::Done,
                next: continuation.next,
            },
            ContinuationChoice::Walk { ref env } => match expr {
                Value::Int(_) => todo!(),
                Value::Str(_) => todo!(),
                Value::Null => todo!(),
                Value::Lambda {
                    param_names: _,
                    body: _,
                } => todo!(),
                Value::Id(id) => {
                    let new_expr = env
                        .get_symbol(&id)
                        .ok_or(RuntimeError::UnresolvedSymbol(id))?
                        .clone();
                    expr = new_expr;
                    continue;
                }
                Value::Match {
                    subject: _,
                    pattern_exprs: _,
                } => todo!(),
                Value::Callsite {
                    function,
                    arguments,
                } => {
                    // Evaluate the callee, then apply the arguments to it.
                    expr = *function;
                    // Chain the continuation.
                    continuation.next = Some(Box::new(Continuation {
                        message: "callsite walk".to_string(),
                        choice: ContinuationChoice::Callsite {
                            env: env.clone(),
                            arguments,
                        },
                        next: continuation.next,
                    }));
                    // Re-use the existing continuation for perf.
                    continue;
                }
                Value::Tuple { dims: _ } => todo!(),
                Value::Thunk { env: _, expr: _ } => todo!(),
                Value::Builtin(_f) => todo!(),
                Value::Let {
                    name: _,
                    value: _,
                    body: _,
                } => todo!(),
                Value::Ctor { name: _, dims: _ } => todo!(),
            },
            ContinuationChoice::Match { .. } => todo!(),
            ContinuationChoice::Callsite { .. } => todo!(),
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
