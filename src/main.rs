mod ast;
mod error;
mod interpreter;
mod loxtype;
mod native_fns;
mod parser;
mod resolver;
mod scanner;
mod token;

use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::Parser as ClapParser;
use rustyline::{error::ReadlineError, DefaultEditor};

use interpreter::Interpreter;
pub(crate) use loxtype::{LoxCallable, LoxType};
pub(crate) type Result<T> = std::result::Result<T, error::Error>;

#[derive(ClapParser)]
struct Cli {
    source_file: Option<PathBuf>,
}

fn run_prompt(interpreter: Interpreter) -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline: std::result::Result<_, _> = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                interpreter.run(&line)?;
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                return Err(anyhow!(err));
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let interpreter = Interpreter::new();

    if let Some(source_file) = cli.source_file {
        let source = fs::read_to_string(source_file)?;
        interpreter.run(&source)?;
    } else {
        run_prompt(interpreter)?;
    }

    Ok(())
}
