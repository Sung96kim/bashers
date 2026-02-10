# Bashers
Installable CLI command helpers (written in Rust)

## Installation

Requires [Rust](https://rustup.rs/) (cargo).

### From crates.io (recommended)

```bash
cargo install bashers
```

Both `bashers` and `bs` are installed to `~/.cargo/bin`. Ensure `~/.cargo/bin` is in your PATH and that it comes **before** any other path that might provide a different `bashers` (e.g. pyenv shims):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### From source

```bash
cargo install --path .
```

Or from git:

```bash
cargo install --git https://github.com/Sung96kim/bashers.git
```

### Manual build

```bash
git clone https://github.com/Sung96kim/bashers.git
cd bashers
cargo build --release
# Binary at target/release/bashers; copy to ~/.cargo/bin/ or /usr/local/bin/
```

## Usage

Both `bashers` and `bs` are available. Run `bashers` or `bashers --help` for the full command list.

```bash
bashers update              # update deps (optional: package name with fuzzy match)
bashers setup               # install project deps (--frozen, --rm, --dry-run)
bashers show                # list installed packages
bashers gh                  # git home: checkout default branch, pull, fetch (--dry-run)
bashers kube kmg <pattern>   # describe pods, show Image lines
bashers kube track <pattern> # follow logs (--err-only, --simple)
bashers self update         # update bashers to latest
bashers version             # print version
```

Verify you're running the Rust binary (e.g. not a pyenv shim):

```bash
which bashers   # expect ~/.cargo/bin/bashers
which bs        # expect ~/.cargo/bin/bs
```

## Commands

| Command | Description |
|---------|-------------|
| **update** | Update Python dependencies (uv/poetry) with fuzzy package matching |
| **setup** | Install project dependencies (uv/poetry) |
| **show** | List installed packages (uv/poetry) |
| **gh** | Git home: checkout default branch, pull, fetch all |
| **kube** | Kubernetes helpers: `kmg` (describe pods / Image), `track` (follow logs) |
| **self** | Self-management: `update` (upgrade bashers) |
| **version** | Print version |

Use `bashers <command> --help` for options.

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

**Quick summary:** Releases are automated via release-plz on push to main: version/changelog PR, merge, then publish to crates.io and create a GitHub Release.
