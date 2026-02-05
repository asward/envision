mod cli;
mod commands;
mod output;
mod session;
mod storage;

use cli::{Cli, Command, SessionAction};
use output::Output;
use std::process;

fn main() {
    let out = Output::new();
    let args = Cli::parse_filtered();

    let result = match args.command {
        Command::Session { action } => match action {
            SessionAction::Init { force, resume } => commands::init::run(&out, force, resume),
        },
        Command::Status => Err("'status' is not yet implemented".into()),
        Command::Set { .. } => Err("'set' is not yet implemented".into()),
        Command::Unset { .. } => Err("'unset' is not yet implemented".into()),
        Command::Clear { .. } => Err("'clear' is not yet implemented".into()),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(msg) => {
            out.error(&format!("Error: {msg}"));
            process::exit(1);
        }
    }
}
