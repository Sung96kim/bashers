# Test Coverage

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

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

- **59.60%** overall coverage (236/396 lines)
- **87 unit tests** passing
- **9 integration tests** passing
- **Total: 96 tests**

### Coverage by Module

| Module | Coverage | Lines Covered | Change |
|--------|----------|---------------|--------|
| `commands/help.rs` | 100% | 38/38 | — |
| `utils/colors.rs` | 96.6% | 28/29 | — |
| `utils/project.rs` | 96.4% | 27/28 | +43.49% |
| `commands/setup.rs` | 78.6% | 55/70 | +23.67% |
| `utils/packages.rs` | 70.5% | 62/88 | — |
| `commands/gh.rs` | 10.8% | 4/37 | +10.81% |
| `commands/show.rs` | 9.7% | 6/62 | +3.01% |
| `commands/update.rs` | 36.4% | 16/44 | — |

## CI Integration

Coverage is automatically calculated in CI using `cargo-tarpaulin`.

## Improving Coverage

To improve coverage:

1. Add unit tests for command modules (especially `gh.rs` and `show.rs`)
2. Add tests for error paths and edge cases
3. Mock external commands for better testability
4. Add integration tests that exercise full command flows
