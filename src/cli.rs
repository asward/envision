use clap::{ColorChoice, CommandFactory, Parser, Subcommand, ValueEnum};

/// Detect --no-color from raw args (before clap parses).
/// Returns true if --no-color flag is present or NO_COLOR env var is set.
pub fn should_disable_color() -> bool {
    std::env::args().any(|a| a == "--no-color") || std::env::var("NO_COLOR").is_ok()
}

/// Filter --no-color from args so clap doesn't see it as an unknown arg
/// when it appears after built-in subcommands like `help`.
pub fn filtered_args() -> Vec<String> {
    std::env::args().filter(|a| a != "--no-color").collect()
}

#[derive(Parser)]
#[command(
    name = "envision",
    about = "See your environment clearly. Change it with precision.",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn parse_filtered() -> Self {
        let color = if should_disable_color() {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };

        let matches = Self::command()
            .color(color)
            .get_matches_from(filtered_args());

        <Self as clap::FromArgMatches>::from_arg_matches(&matches)
            .unwrap_or_else(|e| e.exit())
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage tracking sessions
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Display session status (exits 0 if clean, 1 if dirty)
    Status,

    /// Set and track an environment variable
    Set {
        /// Variable name
        var: String,
        /// Variable value
        value: String,
    },

    /// Unset and track removal of a variable
    Unset {
        /// Variable name
        var: String,
    },

    /// Load environment variables from a profile script
    Profile {
        /// Path to the profile file (.profile.sh or .envision)
        path: String,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Show what would change without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Print shell hook to stdout (add `eval "$(envision hook bash)"` to your RC file)
    Hook {
        /// Shell type
        shell: Shell,
    },

    /// Remove all tracked changes, restore to baseline
    Clear {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Create baseline snapshot of current environment state
    Init {
        /// Reinitialize even if a session already exists (loses tracking history)
        #[arg(long)]
        force: bool,

        /// Resume an existing session instead of creating a new one
        #[arg(long, conflicts_with = "force")]
        resume: bool,
    },
}
