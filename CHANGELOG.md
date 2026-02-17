# CHANGELOG

## [0.8.4](https://github.com/Sung96kim/bashers/compare/v0.8.3...v0.8.4) - 2026-02-17

### Fixed

- python sdist

## [0.8.3](https://github.com/Sung96kim/bashers/compare/v0.8.2...v0.8.3) - 2026-02-17

### Fixed

- release python supported wheels ([#22](https://github.com/Sung96kim/bashers/pull/22))

## [0.8.2](https://github.com/Sung96kim/bashers/compare/v0.8.1...v0.8.2) - 2026-02-17

### Added

- Python wheel builds ([#20](https://github.com/Sung96kim/bashers/pull/20))

## [0.8.1](https://github.com/Sung96kim/bashers/compare/v0.8.0...v0.8.1) - 2026-02-11

### Fixed

- update build to default to base dockerfile

## [0.8.0](https://github.com/Sung96kim/bashers/compare/v0.7.0...v0.8.0) - 2026-02-11

### Added

- Docker cli ([#17](https://github.com/Sung96kim/bashers/pull/17))

## [0.7.0](https://github.com/Sung96kim/bashers/compare/v0.6.0...v0.7.0) - 2026-02-11

### Added

- refactor tui, add watch cli ([#15](https://github.com/Sung96kim/bashers/pull/15))

## [0.6.0](https://github.com/Sung96kim/bashers/compare/v0.5.0...v0.6.0) - 2026-02-10

### Added

- cleanup and add git subcommands

## [0.5.0](https://github.com/Sung96kim/bashers/compare/v0.4.14...v0.5.0) - 2026-02-10

### Added

- `kube kmg`: spinner with completion message "✓ Retrieved images for <pattern>"
- `scripts/install.sh`: runs `cargo install bashers` and adds `~/.cargo/bin` to shell config when needed (e.g. pyenv)
- `scripts/local.sh`: run local build via `cargo run --bin bashers -- <command>`

### Changed

- `scripts/setup-local.sh`: only check, build, test so `./scripts/local.sh` works; removed release archive and install steps
- `setup-local.sh`: version in message now derived from Cargo.toml via `cargo pkgid`

### Fixed

- `kube kmg`: spinner no longer interleaves with output; results collected then printed after spinner clears

## [0.4.15](https://github.com/Sung96kim/bashers/compare/v0.4.14...v0.4.15) - 2026-02-10

### Added

- install script, kmg spinner

## [0.4.14](https://github.com/Sung96kim/bashers/compare/v0.4.13...v0.4.14) - 2026-02-10

### Fixed

- more vibrant colors

## [0.4.13](https://github.com/Sung96kim/bashers/compare/v0.4.12...v0.4.13) - 2026-02-10

### Fixed

- release plz pr step depends

## [0.4.12](https://github.com/Sung96kim/bashers/compare/v0.4.11...v0.4.12) - 2026-02-10

### Added

- Rust

## [0.4.11](https://github.com/Sung96kim/bashers/compare/v0.4.10...v0.4.11) - 2026-02-10

### Added

- Rust
- dependabot
- Rust, fix releases

### Fixed

- use releaseplz releasing
- release again
- remove unused ci

### Other

- *(deps)* bump actions/checkout from 4 to 6 ([#5](https://github.com/Sung96kim/bashers/pull/5))

## v0.4.10 (2026-01-15)

### Bug Fixes

- Logging
  ([`ef311be`](https://github.com/Sung96kim/bashers/commit/ef311be3753945c31c110f9ca35f2a0b3b966d9b))


## v0.4.9 (2026-01-15)

### Bug Fixes

- Python check
  ([`bd0ca1b`](https://github.com/Sung96kim/bashers/commit/bd0ca1b793a963495f1ddd81738ddbee6bb36827))


## v0.4.8 (2026-01-15)

### Bug Fixes

- Local var
  ([`b1f39c6`](https://github.com/Sung96kim/bashers/commit/b1f39c602605734c2def05836e32fb46db56b68e))


## v0.4.7 (2026-01-15)

### Bug Fixes

- Update lib matching, python check
  ([`fcf6848`](https://github.com/Sung96kim/bashers/commit/fcf6848d39a719934a5dfdccd77592a984108da4))


## v0.4.6 (2026-01-13)

### Bug Fixes

- Add version command
  ([`49c9a5e`](https://github.com/Sung96kim/bashers/commit/49c9a5eaaadf52fcfc04c65507a8f793aee2db82))


## v0.4.5 (2026-01-13)

### Bug Fixes

- Disable spinner for now
  ([`a055b76`](https://github.com/Sung96kim/bashers/commit/a055b76afcefb9c593ca2f9998dbcfa5d129df68))


## v0.4.4 (2026-01-13)

### Bug Fixes

- Loader with fzf interaction
  ([`70dce37`](https://github.com/Sung96kim/bashers/commit/70dce37fbe668e15c27535d88426219d5fe588a3))


## v0.4.3 (2026-01-13)

### Bug Fixes

- Loader again
  ([`16c92d2`](https://github.com/Sung96kim/bashers/commit/16c92d231017bf23e1aa2e35dbeb71730fb8809b))


## v0.4.2 (2026-01-13)

### Bug Fixes

- Loader
  ([`dc58145`](https://github.com/Sung96kim/bashers/commit/dc5814523d25aaa21bda6e165525ecd41e723923))


## v0.4.1 (2026-01-13)

### Bug Fixes

- Infinite loop
  ([`b26aaac`](https://github.com/Sung96kim/bashers/commit/b26aaac6474d0fd292ab1baf86467d144579c3ce))


## v0.4.0 (2026-01-13)

### Features

- Github scripts
  ([`b889fb6`](https://github.com/Sung96kim/bashers/commit/b889fb687998ebd7b25cf71dd05e1a3fe9e7f2ce))


## v0.3.0 (2026-01-13)

### Features

- Bash auto completions
  ([`1239813`](https://github.com/Sung96kim/bashers/commit/12398133a59e4d8e8d06006963e948a2f86298e3))


## v0.2.4 (2026-01-13)

### Bug Fixes

- Bashers version
  ([`14d6c37`](https://github.com/Sung96kim/bashers/commit/14d6c37ca13fc59e5052e0d2703e0b90cbf7487d))


## v0.2.3 (2026-01-12)

### Bug Fixes

- Loader cleanup
  ([`21cd06a`](https://github.com/Sung96kim/bashers/commit/21cd06ad2d98a570f8130c0398ef8b2e4db5e663))


## v0.2.2 (2026-01-12)

### Bug Fixes

- Command path
  ([`d520915`](https://github.com/Sung96kim/bashers/commit/d5209150a31aa4942be0a5347c89b46bd8dc68ac))


## v0.2.1 (2026-01-12)

### Bug Fixes

- Path resolution
  ([`59e9e75`](https://github.com/Sung96kim/bashers/commit/59e9e75d8974275d4588fd20a42a19c546194630))


## v0.2.0 (2026-01-12)

### Features

- Colorized help text, dry runs
  ([`5e07058`](https://github.com/Sung96kim/bashers/commit/5e0705854ecb2b8f7d97d90420735773fbb79dad))


## v0.1.1 (2026-01-12)

### Bug Fixes

- Readme
  ([`07568c5`](https://github.com/Sung96kim/bashers/commit/07568c549fda9df79cb175401959d20e919c3ab6))


## v0.1.0 (2026-01-12)

### Features

- Project util scripts
  ([`1423f47`](https://github.com/Sung96kim/bashers/commit/1423f47639d8c76f0948c4e935542023fdbf67cb))


## v0.0.0 (2026-01-12)
