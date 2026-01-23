# Test Coverage

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run with output
cargo test -- --nocapture
```

## Coverage

We use `cargo-tarpaulin` for code coverage.

### Install

```bash
cargo install cargo-tarpaulin --locked
```

### Run Coverage

```bash
# Generate coverage report
cargo tarpaulin --out Xml --output-dir coverage --timeout 120

# View HTML report (if generated)
open coverage/tarpaulin-report.html
```

### Current Coverage

- **45.96%** overall coverage (267/581 lines)
- **114 unit tests** passing
- **0 integration tests** (removed in favor of unit tests)

### Coverage by Module

| Module | Coverage | Lines Covered | Change |
|--------|----------|---------------|--------|
| `commands/help.rs` | 100% | 38/38 | — |
| `utils/colors.rs` | 90.3% | 28/31 | — |
| `utils/project.rs` | 96.2% | 25/26 | — |
| `commands/setup.rs` | 75.3% | 55/73 | — |
| `utils/packages.rs` | 85.0% | 79/93 | — |
| `commands/gh.rs` | 7.1% | 4/56 | — |
| `commands/show.rs` | 9.7% | 6/62 | — |
| `commands/update.rs` | 25.0% | 16/64 | — |
| `commands/self_cmd/update.rs` | 3.9% | 3/77 | — |
| `utils/spinner.rs` | 21.3% | 13/61 | — |

## CI Integration

Coverage is automatically calculated in CI using `cargo-tarpaulin`.

## Improving Coverage

To improve coverage:

1. Add unit tests for command modules (especially `gh.rs`, `show.rs`, and `self_cmd/update.rs`)
2. Add tests for error paths and edge cases
3. Mock external commands for better testability (using dependency injection)
4. Test parsing logic for git commands and GitHub API responses
