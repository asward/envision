# envision

**See your environment clearly. Change it with precision.**

`envision` is a lightweight CLI tool for managing environment variables in POSIX-compatible shells. It tracks what you've changed, shows what changed outside your control, and lets you restore to a known state instantly.

## Why envision?

Working in the terminal, you constantly modify environment variables—setting project paths, toggling debug flags, adjusting configurations. But tracking what you changed vs. what some script changed vs. what was there originally? Impossible.

`envision` solves this by:

- **Tracking your changes** explicitly (set, unset operations)
- **Detecting external changes** (things modified outside the tool)
- **Maintaining a baseline** you can always return to
Think of it as version control for your shell environment.

## Features

- **Baseline snapshots** - Capture your environment's initial state
- **Change tracking** - Know exactly what you modified
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

Add this line to your shell RC file (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
eval "$(envision hook bash)"
```

For fish shell, add to `~/.config/fish/config.fish`:

```fish
envision hook fish | source
```

This sets up a shell wrapper so that commands like `set`, `unset`, `clear`, and `profile` can modify your current shell's environment.

## Usage

### Initialize Tracking

Start tracking in your current shell session:

```bash
envision session init
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
envision status
# Session: abc123
# Baseline: 2024-02-05 14:23:15 UTC
# Tracked changes: 2
# Untracked changes: 0
# Total changed: 2
# State: clean
```

Exit code is 0 when clean (no untracked changes), 1 when dirty.

### Unset Variables

```bash
envision unset DEBUG_MODE
# Unset DEBUG_MODE (was: true)
```

### Load a Profile

Apply environment variables from a profile script:

```bash
# Create a profile
cat > dev.profile.sh <<'EOF'
export DATABASE_URL="postgres://localhost/dev"
export LOG_LEVEL="debug"
export APP_ENV="development"
EOF

# Load it (prompts for confirmation on first use)
envision profile dev.profile.sh
# Loading profile: /home/user/project/dev.profile.sh
# Continue? [y/N] y
# Profile 'dev' loaded
# Variables changed: 3

# Preview without applying
envision profile --dry-run dev.profile.sh

# Skip confirmation
envision profile --yes dev.profile.sh
```

Profile files must use `.profile.sh` or `.envision` extension. Changes are tracked in the active session if one exists.

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
| `envision hook <shell>`      | Print shell integration code                 |
| `envision session init`      | Create baseline snapshot for current session |
| `envision status`            | Show current state and change summary        |
| `envision set <VAR> <value>` | Set and track an environment variable        |
| `envision unset <VAR>`       | Unset and track removal of a variable        |
| `envision profile <file>`    | Load environment variables from a profile    |
| `envision clear`             | Remove all tracked changes, restore baseline |

## How It Works

1. **Initialization** - `envision session init` captures your current environment as the baseline
2. **Tracking** - Each `set`/`unset` operation is recorded in session metadata
3. **Categorization** - Variables are classified as:
   - **Original**: Present in baseline, unchanged
   - **Tracked**: Modified through `envision` commands
   - **Untracked**: Changed outside `envision` (manual exports, scripts)
4. **Restoration** - `clear` removes tracked changes, leaving original and untracked variables intact

## Storage

Session data is stored entirely in the `ENVISION_SESSION` environment variable as base64-encoded JSON. No files are written to disk. The session persists naturally within your shell and is isolated per shell instance.

## Development

```bash
# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- session init

# Check formatting
cargo fmt --check

# Lint
cargo clippy
```

## Roadmap

- [x] Profile system for reusable environment configurations
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
