# Development Guide

## Quick Start

```bash
# Clone and build
git clone <repo-url>
cd bashers
cargo build --release

# Run tests
cargo test

# Try it out
./target/release/bashers --help
```

## Building

### Debug Build
```bash
cargo build
# Binary: target/debug/bashers
```

### Release Build
```bash
cargo build --release
# Binary: target/release/bashers
```

### Install Locally
```bash
# Install to ~/.cargo/bin
cargo install --path .

# Or use cargo install --force to overwrite
cargo install --path . --force
```

## Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_test

# Specific test
cargo test test_fuzzy_match_exact
```

### Test with Output
```bash
# Show println! output
cargo test -- --nocapture

# Show test names
cargo test -- --show-output
```

### Test Coverage
```bash
# Install coverage tool
cargo install cargo-tarpaulin --locked

# Generate coverage
cargo tarpaulin --lib --out Xml --output-dir coverage --timeout 120

# View results
cat coverage/cobertura.xml
```

## Code Quality

### Formatting
```bash
# Format all code
cargo fmt

# Check formatting (CI)
cargo fmt --check
```

### Linting
```bash
# Run clippy
cargo clippy

# Treat warnings as errors
cargo clippy -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix
```

### Type Checking
```bash
# Check without building
cargo check

# Check all targets
cargo check --all-targets
```

## Running Commands

### Direct Execution
```bash
# Using cargo run (quiet mode to suppress build output)
cargo run --quiet -- update --dry-run
cargo run --quiet -- setup --dry-run
cargo run --quiet -- show clap
cargo run --quiet -- gh --dry-run

# Using built binary (recommended for testing)
./target/debug/bashers update --dry-run
./target/release/bashers setup
```

### With Environment Variables
```bash
# Disable spinner
NO_SPINNER=1 cargo run -- update

# Custom install directory (for install.sh)
BASHERS_INSTALL_DIR=/tmp/bin cargo run -- update
```

## Project Structure

```
bashers/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── cli.rs            # CLI definitions
│   ├── commands/         # Command implementations
│   │   ├── update.rs
│   │   ├── setup.rs
│   │   ├── show.rs
│   │   ├── gh.rs
│   │   └── help.rs
│   └── utils/            # Utility functions
│       ├── colors.rs
│       ├── project.rs
│       ├── packages.rs
│       └── spinner.rs
├── tests/
│   └── integration_test.rs
├── Cargo.toml            # Dependencies
└── README.md
```

## Adding New Commands

1. **Create command module**: `src/commands/new_command.rs`
2. **Add to CLI**: Update `src/cli.rs` with new command variant
3. **Wire up**: Add handler in `src/main.rs`
4. **Test**: Add tests in the command module
5. **Document**: Update README.md

Example:
```rust
// src/commands/new_command.rs
pub fn run() -> Result<()> {
    println!("New command!");
    Ok(())
}

// src/cli.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    NewCommand,
}

// src/main.rs
Some(bashers::cli::Commands::NewCommand) => {
    bashers::commands::new_command::run()?;
}
```

## Debugging

### Enable Backtraces
```bash
RUST_BACKTRACE=1 cargo run -- update
RUST_BACKTRACE=full cargo test
```

### Verbose Output
```bash
# Verbose cargo output
cargo build --verbose
cargo test --verbose
```

### Debug Build with Symbols
```bash
# Debug build includes symbols
cargo build
# Use with debugger
gdb target/debug/bashers
```

## CI/CD

The project uses GitHub Actions for:
- **CI** (`.github/workflows/ci.yml`): Runs on every push/PR
  - `cargo check`
  - `cargo build`
  - `cargo test`
  - `cargo clippy`
  - `cargo fmt --check`
  - `cargo tarpaulin` (coverage)

- **Release** (`.github/workflows/release.yml`): Runs on version tags
  - Builds release binary
  - Creates GitHub release
  - Uploads install script

## Common Issues

### "command not found: cargo"
Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Tests fail with "No uv/poetry/cargo project found"
Tests that require project detection need to run in a directory with `Cargo.toml` (which this project has).

### Spinner doesn't show
- Check if `NO_SPINNER` is set
- Verify stdout is a TTY: `test -t 1 && echo "TTY" || echo "Not TTY"`

### Coverage tool fails
Make sure `cargo-tarpaulin` is installed: `cargo install cargo-tarpaulin --locked`
