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
    exec::{Continuation, ContinuationChoice},
    value::Value,
};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The file to execute
    file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = std::fs::read_to_string(args.file).unwrap();
    let decls = parser::program_parser(&input)?;
    log::info!("Parsed: {:#?}", decls);
    let mut env = Env::with_builtins();
    for d in decls.1 {
        log::info!("Decl: {:#?}", d);
        env.add_symbol_mut(d.name, d.body);
    }
    let entrypoint = Value::Callsite {
        function: Box::new(Value::Id("main".into())),
        arguments: vec![],
    };
    let result = walk_tree(Env::with_builtins(), entrypoint);
    log::info!("{:#?}", result);
    Ok(())
}

fn walk_tree(env: Env, expr: Value) -> std::result::Result<Value, crate::exec::RuntimeError> {
    // Return a value in WHNF.
    let message = format!("Walk({expr:?})");
    let mut continuation: Continuation = Continuation::walk(env, expr, message);
    loop {
        log::debug!("walk_tree loop on {continuation:?}");
        continuation = match continuation.choice {
            ContinuationChoice::Done { value } => {
                if let Some(next) = continuation.next {
                    (*next.into_inner()).prepare(value)?
                } else {
                    break Ok(value);
                }
            }
            ContinuationChoice::Walk { .. } => todo!(),
            ContinuationChoice::Match { .. } => todo!(),
            ContinuationChoice::Callsite { .. } => todo!(),
            ContinuationChoice::Thunk { .. } => todo!(),
        }
    }
}
/*
    while True:
        if isinstance(continuation, Done):
            done = continuation
            if not done.next:
                break
            continuation = done.next.prepare(done.value)

        if isinstance(continuation, Walk):
            continuation = continuation.advance()

    return continuation.value
*/
