# Testing Documentation for Hydro Repository

## Overview

This document provides comprehensive information about the testing strategy and test coverage for the Hydro project. The tests ensure code quality, correctness, and maintainability across all crates in the workspace.

## Table of Contents

- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Test Coverage by Crate](#test-coverage-by-crate)
- [Writing New Tests](#writing-new-tests)
- [Best Practices](#best-practices)

## Test Organization

The Hydro project follows a consistent testing structure across all crates:

```
crate_name/
├── src/
│   ├── lib.rs
│   └── module.rs      # May contain #[cfg(test)] inline tests
└── tests/
    ├── module_tests.rs
    └── integration_tests.rs
```

### Test Types

1. **Unit Tests**: Located in dedicated `tests/` directories, focusing on individual functions and modules
2. **Integration Tests**: Located in `tests/` directories, testing interactions between modules
3. **Inline Tests**: Marked with `#[cfg(test)]` in source files for testing private functions
4. **Documentation Tests**: Embedded in doc comments, ensuring examples stay up-to-date

## Running Tests

### Run All Tests

```bash
cargo test --workspace
```

### Run Tests for a Specific Crate

```bash
cargo test -p hydro_std
cargo test -p dfir_lang
cargo test -p hydro_build_utils
```

### Run a Specific Test File

```bash
cargo test -p hydro_std --test rolling_average_tests
cargo test -p hydro_std --test membership_tests
cargo test -p dfir_lang --test union_find_tests
```

### Run a Specific Test Function

```bash
cargo test -p hydro_std test_add_multiple_samples
cargo test -p dfir_lang test_basic_union
```

### Run Tests with Output

```bash
cargo test -- --nocapture
cargo test -- --show-output
```

### Run Tests in Release Mode

```bash
cargo test --release
```

## Test Coverage by Crate

### hydro_std

**Location**: `hydro_std/tests/`

**Test Files**:
- `rolling_average_tests.rs` - Tests for statistical calculations (29 tests)
- `membership_tests.rs` - Tests for cluster membership tracking (13 tests)
- `request_response_tests.rs` - Tests for request-response pattern (11 tests)

**Inline Tests**:
- `src/quorum.rs` - Quorum collection logic (7 tests)

**Coverage**:
- ✅ `bench_client/rolling_average.rs` - Comprehensive coverage of mean, variance, std dev, confidence intervals
- ✅ `membership.rs` - Full coverage of join/leave events and state tracking
- ✅ `request_response.rs` - Coverage of metadata-response joins
- ✅ `quorum.rs` - Coverage of quorum collection with various scenarios
- ⚠️ `compartmentalize.rs` - Integration tests needed (requires cluster setup)
- ⚠️ `bench_client/mod.rs` - Integration tests needed (requires full bench setup)

### dfir_lang

**Location**: `dfir_lang/tests/`

**Test Files**:
- `union_find_tests.rs` - Tests for union-find data structure (20 tests)

**Inline Tests**:
- `src/union_find.rs` - Basic union-find test (1 test)

**Coverage**:
- ✅ `union_find.rs` - Comprehensive coverage of union, find, same_set operations
- ⚠️ `parse.rs` - Parsing logic needs test coverage
- ⚠️ `diagnostic.rs` - Diagnostic utilities need test coverage
- ⚠️ `pretty_span.rs` - Span formatting needs test coverage
- ⚠️ `process_singletons.rs` - Singleton processing needs test coverage

### hydro_build_utils

**Location**: `hydro_build_utils/tests/`

**Test Files**:
- `macro_tests.rs` - Tests for build-time macros and utilities (26 tests)

**Coverage**:
- ✅ `lib.rs` - Comprehensive coverage of macro expansion and snapshot testing wrappers
- ✅ Re-exports (rustc_version, insta) - Verified accessible and functional

### Existing Test Coverage

The following crates already have established test suites:

- **dfir_rs**: `tests/` directory with comprehensive test coverage
- **hydro_lang**: `tests/` directory with test coverage
- **lattices**: `tests/` directory with lattice property tests
- **multiplatform_test**: `tests/` directory with cross-platform tests
- **sinktools**: `tests/` directory with sink adaptor tests (build.rs, direct.rs)
- **variadics**: `tests/` directory with variadic tests

## Writing New Tests

### Test Structure Template

```rust
//! Comprehensive unit tests for [module name].
//!
//! This module tests [brief description of what is being tested].

use crate::module_name::FunctionName;

#[test]
fn test_basic_functionality() {
    // Arrange
    let input = create_test_input();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected_output);
}

#[test]
fn test_edge_case_empty_input() {
    // Test empty/zero cases
}

#[test]
fn test_error_handling() {
    // Test error cases
}
```

### Hydro Flow Testing Template

For testing Hydro dataflow operations:

```rust
use hydro_lang::prelude::*;

#[test]
fn test_flow_operation() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = operation_under_test(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send test data
        in_send.send(test_data).unwrap();

        // Verify output
        out_recv.assert_yields_only_unordered(expected_data).await;
    });
}
```

### Naming Conventions

- Test files: `{module_name}_tests.rs`
- Test functions: `test_{functionality}_{scenario}`
- Examples:
  - `test_add_sample_single_value`
  - `test_union_find_basic_operations`
  - `test_membership_join_then_leave`

### Test Categories

1. **Happy Path Tests**: Test normal, expected usage
2. **Edge Case Tests**: Empty inputs, boundary values, single elements
3. **Error Tests**: Invalid inputs, error conditions
4. **Integration Tests**: Multiple components working together
5. **Property Tests**: Verify invariants (e.g., commutativity, associativity)

## Best Practices

### General Guidelines

1. **One Assertion per Test**: Keep tests focused on a single behavior
2. **Descriptive Names**: Test names should clearly describe what is being tested
3. **Arrange-Act-Assert**: Structure tests in three clear phases
4. **Independent Tests**: Tests should not depend on each other
5. **Fast Execution**: Keep tests fast; use mocks for external dependencies

### Hydro-Specific Guidelines

1. **Use Exhaustive Simulation**: Use `flow.sim().exhaustive()` for deterministic testing
2. **Test Ordering Assumptions**: Explicitly test ordering requirements with appropriate stream types
3. **Test Tick Boundaries**: Test behavior across tick boundaries when relevant
4. **Use Appropriate Assertions**: 
   - `assert_yields_only_unordered()` for unordered results
   - `assert_yields_only()` for ordered results
   - `assert_no_more()` to verify empty output

### Coverage Goals

- **Line Coverage**: Aim for >80% line coverage
- **Branch Coverage**: Test all conditional branches
- **Edge Cases**: Cover boundary conditions and empty inputs
- **Error Paths**: Test error handling and recovery

### Documentation

1. **Module-level Docs**: Each test file should have a module-level doc comment
2. **Test Comments**: Complex test logic should have explanatory comments
3. **Example Tests**: Provide examples of correct usage in doc tests

## Continuous Integration

Tests are run automatically on:
- Every pull request
- Commits to main branches
- Release candidates

### CI Commands

```bash
# Run all tests
cargo test --workspace --all-features

# Run tests with additional checks
cargo clippy --workspace --all-features
cargo fmt --check

# Run specific test suites
cargo test -p hydro_std
cargo test -p dfir_lang
cargo test -p hydro_build_utils
```

## Test Metrics

### Current Coverage Summary

| Crate | Test Files | Test Count | Coverage Status |
|-------|-----------|------------|-----------------|
| hydro_std | 3 | 53 | ✅ Good |
| dfir_lang | 1 | 20 | ⚠️ Partial |
| hydro_build_utils | 1 | 26 | ✅ Good |
| sinktools | 2 | Many | ✅ Good |
| variadics | Multiple | Many | ✅ Good |
| lattices | Multiple | Many | ✅ Good |

### Areas for Improvement

1. **dfir_lang**: Add tests for parse, diagnostic, and process_singletons modules
2. **hydro_std**: Add integration tests for compartmentalize and bench_client modules
3. **Property-based Testing**: Consider adding proptest for complex data structures
4. **Performance Tests**: Add benchmarks in `benches/` for critical paths

## Troubleshooting

### Common Issues

1. **Test Timeout**: Increase timeout with `#[tokio::test(flavor = "multi_thread", worker_threads = 1)]`
2. **Non-deterministic Failures**: Use exhaustive simulation or fix non-deterministic code
3. **Async Test Issues**: Ensure proper use of `async`/`await` and runtime setup

### Debugging Tests

```bash
# Run test with verbose output
cargo test test_name -- --nocapture --test-threads=1

# Run test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run test with full backtrace
RUST_BACKTRACE=full cargo test test_name
```

## Contributing Tests

When contributing new tests:

1. Follow the existing test structure and naming conventions
2. Add module-level documentation
3. Ensure tests are independent and deterministic
4. Update this document if adding new test patterns or conventions
5. Run the full test suite before submitting PR: `cargo test --workspace`

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Insta Snapshot Testing](https://insta.rs/)

## Contact

For questions about testing:
- Check existing test files for examples
- Review this documentation
- Ask in team discussions or code reviews
