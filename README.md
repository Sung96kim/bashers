# Bashers

CLI command helpers in Rust. Install as `bashers` or `bs`; both binaries go to the same place.

## Installation

- **Cargo:** `cargo install bashers`  
  Puts `bashers` and `bs` in `~/.cargo/bin`. Ensure that directory is in PATH (and before pyenv/shims if you use pyenv).

- **PyPI:** `pip install bashers` or `pip install --upgrade bashers`  
  Wheels for Python 3.11, 3.12, 3.13. If you're on 3.13 and see an old version, a 3.13 wheel may not exist yet—use the next release, or install from Cargo/repo.

- **From repo:**  
  `./scripts/install.sh`  
  Or: `curl -sSf https://raw.githubusercontent.com/Sung96kim/bashers/main/scripts/install.sh | sh`  
  Use `--no-path` to skip profile changes.

## Usage

```bash
bashers update                    # deps (optional packages; fuzzy match; -v verbose, -y auto-select)
bashers update -v pkg1 pkg2       # selected packages, show tool output at end
bashers setup                     # install deps (--frozen, --rm, --dry-run)
bashers show                      # list packages
bashers git sync                  # default branch, pull, fetch (--current = current branch only)
bashers kube kmg <pattern>        # pod describe + Image lines
bashers kube track <pattern>      # follow logs (--err-only, --simple)
bashers docker build [-f <path>]  # Dockerfile (default ./Dockerfile; -t tag, --no-cache, -c context)
bashers watch -n 2 -- <cmd>       # run repeatedly, highlight changes (-n interval, --no-diff)
bashers self update               # upgrade bashers
bashers version
bashers --gui                     # open desktop GUI (requires --features gui)
```

`bs` works as an alias for `bashers` (e.g. `bs sync`, `bs build`). Run `bashers <cmd> --help` for options.

### Commands

| Command   | Description |
| --------- | ----------- |
| **update** | Deps (cargo/uv/poetry). Optional package names (fuzzy match, multi-select). `-v` show tool output at end, `-y` auto-select. |
| **setup**  | Install project deps. |
| **show**   | List installed packages. |
| **git**    | `sync` (default branch or `--current`). |
| **kube**   | `kmg`, `track`. |
| **docker** | `build` (optional Dockerfile path, tag, no-cache, context). |
| **watch**  | Run on an interval, diff highlight (green = changed). |
| **self**   | `update`. |
| **version** | Print version. |
| **--gui** | Launch desktop GUI (requires `cargo install bashers --features gui`). |

## Features

- Fuzzy package matching; multi-select when multiple matches
- cargo, uv & poetry
- Color output, dry-run

## Development

### Build & test

```bash
cargo build
cargo build --release
cargo test
```

Run a single test: `cargo test test_fuzzy_match_exact`

### Code quality

```bash
cargo fmt
cargo clippy -- -D warnings
```

### Running locally

| Method | Command |
|--------|--------|
| Via cargo | `cargo run --quiet -- <cmd>` |
| Binary | `./target/debug/bashers <cmd>` |
| No install (script) | `./scripts/local.sh <cmd>` |

Set `NO_SPINNER=1` to disable the spinner.

### Scripts (no install)

- **`./scripts/local.sh <cmd>`** — Runs the repo binary via `cargo run --bin bashers`. Example: `./scripts/local.sh update`.
- **`./scripts/setup-local.sh`** — Runs `cargo check`, `cargo build`, `cargo test`, then reminds you to use `./scripts/local.sh`.

### Coverage

```bash
cargo install cargo-tarpaulin --locked
cargo tarpaulin --out Xml --output-dir coverage --timeout 120
```

### Python wheel (build & test)

1. Install [maturin](https://pypi.org/project/maturin/): `pip install maturin`
2. From repo root, build: `maturin build --release --features pyo3`  
   Wheels are written to `target/wheels/`.
3. Install with a matching Python (e.g. 3.13):

   ```bash
   python3 -m pip install --force-reinstall target/wheels/bashers-*-cp313-*.whl
   ```

4. Confirm: `bashers --help` or `bashers version`

### Adding a new command

1. Add a module under `src/commands/` (or `src/commands/<group>/`).
2. Add the variant in `src/cli.rs` and wire it in `src/lib.rs`.
3. Run `cargo build`.
4. Update the **Usage** section and **Commands** table in this README.

## Releasing

**Automated (release-plz):** On push to main, use [Conventional Commits](https://www.conventionalcommits.org/): `feat:` (minor), `fix:` (patch), `feat!:` or `BREAKING CHANGE:` (major). Push → version/changelog PR → merge → publish to crates.io and GitHub Release. The tag and GitHub Release are created in the workflow run triggered by the merge. Set `CARGO_REGISTRY_TOKEN` for crates.io. First time: run `cargo publish` once so release-plz knows the current version.

**Manual:** Bump version in `Cargo.toml`, tag `vX.Y.Z`, push tag; workflow builds and creates the GitHub Release.
