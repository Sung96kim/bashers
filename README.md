# Bashers

CLI command helpers (Rust). Install: `cargo install bashers`. Both `bashers` and `bs` go to `~/.cargo/bin`; ensure it’s in PATH before pyenv/shims.

**Install from repo:** `./scripts/install.sh` (or `curl -sSf https://raw.githubusercontent.com/Sung96kim/bashers/main/scripts/install.sh | sh`). Use `--no-path` to skip profile changes.

## Usage

```bash
bashers update              # update deps (fuzzy match optional)
bashers setup               # install deps (--frozen, --rm, --dry-run)
bashers show                # list packages
bashers git sync            # checkout default, pull, fetch (--current = current branch only); bs sync works
bashers kube kmg <pattern>  # pod describe + Image lines
bashers kube track <pattern> # follow logs (--err-only, --simple)
bashers docker build [ -f <path> ]  # build from Dockerfile (default: ./Dockerfile; -t tag, --no-cache, -c context); bs build works
bashers watch -n 2 -- <cmd> # run command repeatedly, highlight changes (-n interval, --no-diff)
bashers self update         # upgrade bashers
bashers version
```

| Command | Description |
|---------|-------------|
| **update** | Update deps (uv/poetry), fuzzy match |
| **setup** | Install project deps |
| **show** | List installed packages |
| **git** | `sync` (default branch or --current) |
| **kube** | `kmg`, `track` |
| **docker** | `build` (optional Dockerfile path [default: ./Dockerfile], tag, no-cache, context) |
| **watch** | Run command on an interval, diff highlight (green = changed) |
| **self** | `update` |
| **version** | Print version |

`bashers <cmd> --help` for options.

## Features

Fuzzy package matching, fzf when multiple matches, uv & poetry, color output, dry-run.

## Development

**Build:** `cargo build` / `cargo build --release`

**Test:** `cargo test` (unit: `cargo test --lib`; one test: `cargo test test_fuzzy_match_exact`)

**Quality:** `cargo fmt` · `cargo clippy` · `cargo fmt --check` · `cargo clippy -- -D warnings`

**Run:** `cargo run --quiet -- <cmd>` or `./target/debug/bashers <cmd>`. Optional: `NO_SPINNER=1` to disable spinner.

**Coverage:** `cargo install cargo-tarpaulin --locked` then `cargo tarpaulin --out Xml --output-dir coverage --timeout 120`

**New command:** Add module under `src/commands/` (or `src/commands/<group>/`), add variant in `src/cli.rs`, wire in `src/main.rs`, then `cargo build`. When adding or changing any CLI command, update the Usage section and the Command table above.

## Releasing

Releases are automated with **release-plz** on push to main. Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:` (minor), `fix:` (patch), `feat!:` or `BREAKING CHANGE:` (major). Push to main → version/changelog PR, merge → publish to crates.io and GitHub Release. The **tag and GitHub Release are created in the workflow run triggered by the merge** (not the run that opened the PR). Set `CARGO_REGISTRY_TOKEN` if publishing to crates.io. First time: run `cargo publish` once so release-plz knows the current version.

Manual: bump version in `Cargo.toml`, tag `vX.Y.Z`, push tag; workflow builds and creates the GitHub Release.
