use error::Result;

mod env;
mod error;
mod exec;
mod parser;
mod runtime;
mod value;

// Example clap arguments.
use crate::{
    env::Env,
    exec::{is_weak_head_normal_form, Continuation, ContinuationChoice, RuntimeError},
    value::Value,
};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The file to execute
    file: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let input = std::fs::read_to_string(args.file).unwrap();
    let decls = parser::program_parser(&input)?;
    let mut env = Env::with_builtins();
    for d in decls.1 {
        tracing::info!("{:#?}", d);
        env.add_symbol_mut(d.name, d.body);
    }
    let entrypoint = Value::Callsite {
        function: Box::new(Value::Id("main".into())),
        arguments: vec![],
    };
    let result = walk_tree(Env::with_builtins(), entrypoint);
    tracing::info!("{:#?}", result);
    Ok(())
}

fn walk_tree(env: Env, mut expr: Value) -> std::result::Result<Value, RuntimeError> {
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
                Value::Id(_id) => todo!(),
                Value::Match {
                    subject: _,
                    pattern_exprs: _,
                } => todo!(),
                Value::Callsite {
                    function,
                    arguments,
                } => {
                    // Evaluation the function, then pass that to the arguments.
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
