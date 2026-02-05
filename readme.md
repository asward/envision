# envision

**See your environment clearly. Change it with precision.**

`envision` is a lightweight CLI tool for managing environment variables in POSIX-compatible shells. It tracks what you've changed, shows what changed outside your control, and lets you restore to a known state instantly.

## Why envision?

Working in the terminal, you constantly modify environment variables—setting project paths, toggling debug flags, adjusting configurations. But tracking what you changed vs. what some script changed vs. what was there originally? Impossible.

`envision` solves this by:

- **Tracking your changes** explicitly (set, unset operations)
- **Detecting external changes** (things modified outside the tool)
- **Maintaining a baseline** you can always return to
- **Showing clear diffs** of your environment state

Think of it as version control for your shell environment.

## Features

- **Baseline snapshots** - Capture your environment's initial state
- **Change tracking** - Know exactly what you modified
- **Smart diffing** - See original vs. tracked vs. untracked changes
- **Quick reset** - Return to baseline instantly
- **Session-based** - Per-shell tracking, no global pollution
- **Zero dependencies** - Single static binary

## Installation

### From Source

**Prerequisites:**

- Rust 1.70 or higher ([install via rustup](https://rustup.rs/))

**Build:**

```bash
git clone https://github.com/yourusername/envision.git
cd envision
cargo build --release
```

**Install:**

```bash
# Copy binary to your PATH
cp target/release/envision ~/.local/bin/

# Or use cargo install
cargo install --path .
```

### Shell Integration

For `envision` to modify your current shell's environment, add this function to your shell RC file (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
envision() {
    if [ "$1" = "set" ] || [ "$1" = "unset" ] || [ "$1" = "clear" ]; then
        eval "$(command envision "$@")"
    else
        command envision "$@"
    fi
}
```

This allows `envision` to output shell commands that modify your environment when needed.

## Usage

### Initialize Tracking

Start tracking in your current shell session:

```bash
envision init
# Session initialized: abc123
# Baseline captured: 47 variables
```

### Set Variables

```bash
envision set PROJECT_ROOT /home/user/myproject
# Set PROJECT_ROOT=/home/user/myproject

envision set DEBUG_MODE true
# Set DEBUG_MODE=true
```

### Check Status

```bash
envision status current
# Session: abc123
# Baseline: 2024-02-05 14:23:15
# Tracked changes: 2
# Untracked changes: 0
# State: clean
```

### View Differences

```bash
envision diff show
# + PROJECT_ROOT=/home/user/myproject (tracked)
# + DEBUG_MODE=true (tracked)
# 47 unchanged variables
```

If you modify variables outside `envision`:

```bash
export MANUAL_VAR=foo

envision diff show
# + PROJECT_ROOT=/home/user/myproject (tracked)
# + DEBUG_MODE=true (tracked)
# + MANUAL_VAR=foo (untracked)
# 47 unchanged variables
```

### Unset Variables

```bash
envision unset DEBUG_MODE
# Unset DEBUG_MODE (was: true)
```

### Reset to Baseline

```bash
envision clear
# Preview changes:
#   Remove: PROJECT_ROOT
#   Remove: DEBUG_MODE
# Continue? (y/N): y
# Cleared 2 tracked variables
# State restored to baseline
```

Or skip confirmation:

```bash
envision clear --force
```

## Command Reference

| Command                      | Description                                  |
| ---------------------------- | -------------------------------------------- |
| `envision init`              | Create baseline snapshot for current session |
| `envision status current`    | Show current state and change summary        |
| `envision set <VAR> <value>` | Set and track an environment variable        |
| `envision unset <VAR>`       | Unset and track removal of a variable        |
| `envision clear`             | Remove all tracked changes, restore baseline |
| `envision diff show`         | Display changes from baseline                |

## How It Works

1. **Initialization** - `envision init` captures your current environment as the baseline
2. **Tracking** - Each `set`/`unset` operation is recorded in session metadata
3. **Categorization** - Variables are classified as:
   - **Original**: Present in baseline, unchanged
   - **Tracked**: Modified through `envision` commands
   - **Untracked**: Changed outside `envision` (manual exports, scripts)
4. **Restoration** - `clear` removes tracked changes, leaving original and untracked variables intact

## Storage

Session data is stored in `~/.local/share/envision/sessions/` (or `$XDG_DATA_HOME/envision/sessions/`).

Each shell session gets a unique identifier. Data persists across shell restarts within the same session.

## Development

```bash
# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- init

# Check formatting
cargo fmt --check

# Lint
cargo clippy
```

## Roadmap

- [ ] Profile/template system for reusable environment configurations
- [ ] Interactive TUI for visual state management
- [ ] Snapshot comparison and history
- [ ] Export/import environment configurations
- [ ] Shell prompt integration (show active profile)

## License

MIT

## Contributing

Contributions welcome! Please open an issue to discuss major changes.

---

**Questions?** Open an issue or start a discussion.
