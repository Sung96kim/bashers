# Releasing

This document explains how to create a new release of `bashers`.

## Automated Semantic Release (Recommended)

The project uses **release-plz** for automated semantic versioning based on [Conventional Commits](https://www.conventionalcommits.org/).

### How It Works

1. **Make commits** following conventional commit format:
   - `feat: add new feature` → bumps minor version (0.4.9 → 0.5.0)
   - `fix: fix bug` → bumps patch version (0.4.9 → 0.4.10)
   - `feat!: breaking change` → bumps major version (0.4.9 → 1.0.0)

2. **Push to main** - The release workflow:
   - Runs `release-plz update` to bump the version in `Cargo.toml` and update the changelog from conventional commits
   - Commits and pushes those changes directly to main (with `[skip ci]` to avoid an extra run)
   - Runs `release-plz release` to create the git tag, publish to crates.io (if configured), and create the GitHub Release with the binary attached

So: push to main with conventional commits → version bump and release happen automatically in one run.

**First-time setup:** The package must exist on crates.io so release-plz can determine the current version. If you haven't published yet, run `cargo publish` once from main (with the version you want, e.g. the current `Cargo.toml` version), then the workflow will take over for future releases.

### PyPI Publishing

The `release.yml` workflow uses semantic-release to publish to PyPI on push to main. It requires:
- `PYPI_TOKEN` secret
- `Pypi push` environment

### Conventional Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature (minor bump)
- `fix`: Bug fix (patch bump)
- `docs`: Documentation only
- `style`: Code style changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding tests
- `chore`: Maintenance tasks

**Breaking changes:** Add `!` after type (e.g., `feat!: breaking change`) or include `BREAKING CHANGE:` in footer.

## Manual Release Process

If you prefer manual releases, you can still use the tag-based workflow:

When you push a version tag (e.g., `v0.4.9`), the workflow will:

1. Build the release binary for Linux x86_64
2. Create a `.tar.gz` archive
3. Create a GitHub Release with the binary attached
4. Update the install script with the correct repository name

## Steps to Create a Manual Release

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
version = "0.4.9"  # Update to new version
```

### 2. Commit Changes

```bash
git add Cargo.toml
git commit -m "Bump version to 0.4.9"
```

### 3. Create and Push Tag

Create a tag matching the version (must start with `v`):

```bash
git tag v0.4.9
git push origin v0.4.9
```

Or create an annotated tag with a message:

```bash
git tag -a v0.4.9 -m "Release v0.4.9"
git push origin v0.4.9
```

### 4. Verify Release

After pushing the tag:

1. Check GitHub Actions: https://github.com/Sung96kim/bashers/actions
2. The "Release" workflow should run automatically
3. Once complete, check Releases: https://github.com/Sung96kim/bashers/releases
4. The release should include:
   - `bashers-linux-x86_64.tar.gz` (the binary archive)
   - `scripts/install.sh` (the installation script)

### 5. Test Installation

Test that users can install the release:

```bash
curl -LsSf https://raw.githubusercontent.com/Sung96kim/bashers/main/scripts/install.sh | sh
```

Or test a specific version:

```bash
curl -LsSf https://raw.githubusercontent.com/Sung96kim/bashers/main/scripts/install.sh | sh -s -- 0.4.9
```

## Manual Release (Alternative)

If you need to create a release manually without using the workflow:

```bash
# Build the release binary
cargo build --release

# Create archive
chmod +x target/release/bashers
tar czf bashers-linux-x86_64.tar.gz -C target/release bashers

# Then manually upload to GitHub Releases
```

## Release Workflow Details

The release workflow (`.github/workflows/release.yml`) does the following:

1. **Triggers on**: Push of tags matching `v*` (e.g., `v0.4.9`)
2. **Builds**: Release binary for Linux x86_64
3. **Packages**: Creates `bashers-linux-x86_64.tar.gz`
4. **Releases**: Creates GitHub Release with:
   - Binary archive
   - Install script
   - Release notes with installation instructions

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Example: `v0.4.9` → `v0.4.10` (patch), `v0.5.0` (minor), `v1.0.0` (major)
