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

    let result: Result<u8, String> = match args.command {
        Command::Session { action } => match action {
            SessionAction::Init { force, resume } => commands::session::init(&out, &mut ex, force, resume),
        },
        Command::Profile { path, yes, dry_run } => commands::profile::run(&out, &mut ex, &path, yes, dry_run),
        Command::Hook { shell } => commands::hook::run(&shell),
        Command::Status => commands::status::run(&out),
        Command::Set { var, value } => commands::set::run(&out, &mut ex, &var, &value),
        Command::Unset { var } => commands::unset::run(&out, &mut ex, &var),
        Command::Clear { .. } => Err("'clear' is not yet implemented".into()),
    };

    match result {
        Ok(code) => {
            ex.flush();
            process::exit(code as i32);
        }
        Err(msg) => {
            out.error(&format!("Error: {msg}"));
            process::exit(1);
        }
    }
}
