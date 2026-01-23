# Bashers
Installable CLI command helpers (written in Rust)

## Installation

### Quick Install (Recommended)

Install with a single command:

```bash
curl -LsSf https://raw.githubusercontent.com/Sung96kim/bashers/main/install.sh | sh
```

Or specify a version:

```bash
curl -LsSf https://raw.githubusercontent.com/Sung96kim/bashers/main/install.sh | sh -s -- 0.4.9
```

The script will:
- Download the Linux x86_64 binary from GitHub releases
- Install to `~/.local/bin/bashers`
- Add to PATH if needed

**Note:** Linux x86_64 only. For other platforms/architectures, build from source.

### From Source

```bash
cargo install --path .
```

Or from git:

```bash
cargo install --git https://github.com/Sung96kim/bashers.git
```

### Manual Build

```bash
git clone https://github.com/Sung96kim/bashers.git
cd bashers
cargo build --release
# Binary at target/release/bashers
# Copy to ~/.local/bin/ or /usr/local/bin/
```

## Usage

After installation, use the `bashers` command:

```bash
bashers update
bashers update requests
bashers show
bashers show requests
bashers setup
bashers setup --frozen
bashers setup --rm
bashers gh
bashers gh --dry-run
```

Verify the command is on PATH:

```bash
which bashers
```

The binary will be installed to `~/.cargo/bin/bashers` by default (when using `cargo install`).
Make sure `~/.cargo/bin` is in your PATH.

## Commands

- **update** - Update Python dependencies (uv/poetry) with fuzzy package matching
- **setup** - Install project dependencies (uv/poetry)
- **show** - List installed packages (uv/poetry)
- **gh** - Git home: checkout default branch, pull, fetch all

## Features

- **Fuzzy matching** - Find packages with partial names (e.g., `bashers update indi` matches `indicodata-core`)
- **fzf integration** - Interactive selection when multiple matches found
- **uv & poetry support** - Works with both package managers
- **Color output** - Beautiful colored terminal output
- **Dry-run mode** - Preview commands before executing

## Development

### Prerequisites

- Rust and Cargo installed ([rustup.rs](https://rustup.rs/))
- For testing: `cargo-tarpaulin` (optional, for coverage)

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Binary location
# Debug: target/debug/bashers
# Release: target/release/bashers
```

### Running

```bash
# Run directly with cargo (quiet mode to suppress build output)
cargo run --quiet -- update --dry-run
cargo run --quiet -- setup --dry-run
cargo run --quiet -- show
cargo run --quiet -- gh --dry-run

# Or use the built binary (recommended for testing)
./target/debug/bashers update --dry-run
./target/release/bashers setup --dry-run
```

### Testing

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test --lib test_fuzzy_match_exact
```

### Code Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin --locked

# Generate coverage report
cargo tarpaulin --out Xml --output-dir coverage --timeout 120

# View coverage (if HTML generated)
open coverage/tarpaulin-report.html
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint with clippy
cargo clippy

# Lint with strict warnings
cargo clippy -- -D warnings
```

### Development Workflow

```bash
# 1. Make changes
# 2. Check code compiles
cargo check

# 3. Run tests
cargo test

# 4. Format code
cargo fmt

# 5. Check for issues
cargo clippy

# 6. Build release
cargo build --release
```

## Adding New Commands

1. Add a new command module in `src/commands/`
2. Implement the command function
3. Add the command variant to `src/cli.rs`
4. Wire it up in `src/main.rs`
5. Rebuild: `cargo build`

## Releasing

See [RELEASING.md](RELEASING.md) for instructions on creating a new release.

**Quick summary:** Push a version tag (e.g., `v0.4.9`) to trigger the automated release workflow, which builds the binary and creates a GitHub Release.
