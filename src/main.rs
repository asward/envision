mod cli;
mod commands;
mod output;
mod session;
mod storage;

use cli::{Cli, Command};
use output::Output;
use std::process;

fn main() {
    let out = Output::new();
    let args = Cli::parse_filtered();

    let result = match args.command {
        Command::Init { force, resume } => commands::init::run(&out, force, resume),
        Command::Status { action: _ } => Err("'status' is not yet implemented".into()),
        Command::Manipulate(_) => Err("'manipulate' is not yet implemented".into()),
        Command::Diff { action: _ } => Err("'diff' is not yet implemented".into()),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(msg) => {
            out.error(&format!("Error: {msg}"));
            process::exit(1);
        }
    }
}
