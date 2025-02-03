use error::Result;

mod env;
mod error;
mod exec;
mod parser;
mod runtime;
mod value;

// Example clap arguments.
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
    Ok(())
}
