mod cli;
mod commands;
mod export;
mod output;
mod session;

use cli::{Cli, Command, SessionAction};
use export::Exports;
use output::Output;
use std::process;

fn main() {
    let out = Output::new();
    let mut ex = Exports::new();
    let args = Cli::parse_filtered();

    let result = match args.command {
        Command::Session { action } => match action {
            SessionAction::Init { force, resume } => commands::session::init(&out, &mut ex, force, resume),
        },
        Command::Hook { shell } => commands::hook::run(&shell),
        Command::Status => Err("'status' is not yet implemented".into()),
        Command::Set { var, value } => commands::set::run(&out, &mut ex, &var, &value),
        Command::Unset { var } => commands::unset::run(&out, &mut ex, &var),
        Command::Clear { .. } => Err("'clear' is not yet implemented".into()),
    };

    match result {
        Ok(()) => {
            ex.flush();
            process::exit(0);
        }
        Err(msg) => {
            out.error(&format!("Error: {msg}"));
            process::exit(1);
        }
    }
}
