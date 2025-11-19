# Hydro Testing Guide

This document provides comprehensive information about testing in the Hydro project.

## Test Organization

The Hydro project uses a comprehensive testing strategy with tests organized at multiple levels:

### Unit Tests
Unit tests are located alongside the code they test using Rust's built-in `#[cfg(test)]` modules, or in separate `tests/` directories within each crate.

### Integration Tests
Integration tests are located in:
- Individual crate `tests/` directories
- Workspace-level `tests/` directory for cross-crate integration tests

### Compile-Fail Tests
Many crates include compile-fail tests to ensure that invalid code is properly rejected by the compiler. These are typically in:
- `tests/compile-fail/` - For stable Rust
- `tests/compile-fail-nightly/` - For nightly-specific tests

## Test Coverage by Crate

### Core DFIR Components

#### dfir_rs
- **Location**: `/dfir_rs/tests/`
- **Coverage**: Comprehensive surface API tests, groupby, scheduling, async operations
- **Key Test Files**:
  - `surface_*.rs` - Surface API tests
  - `scheduled_*.rs` - Scheduling tests
  - `groupby.rs` - Grouping operations
  - Compile-fail tests for type safety

#### dfir_lang
- **Location**: `/dfir_lang/src/` (inline) and `/dfir_lang/tests/`
- **Coverage**: Language constructs and code generation

#### dfir_macro
- **Location**: `/dfir_macro/src/` (inline tests)
- **Coverage**: Procedural macro implementations

### Hydro Components

#### hydro_lang
- **Location**: `/hydro_lang/tests/`
- **Coverage**: High-level distributed programming constructs
- **Features**: Simulation and deployment testing

#### hydro_std
- **Location**: `/hydro_std/tests/`
- **NEW**: Comprehensive tests added for:
  - `quorum_tests.rs` - Quorum collection patterns
  - `membership_tests.rs` - Cluster membership tracking
  - `request_response_tests.rs` - Request-response patterns
- **Coverage**: Standard distributed patterns and utilities

#### hydro_test
- **Location**: `/hydro_test/examples/`
- **Coverage**: Integration tests using examples as test cases

#### hydro_deploy
- **Location**: Multiple deployment crates
- **Coverage**: Deployment infrastructure and examples

### Lattice Components

#### lattices
- **Location**: `/lattices/tests/`
- **Tests**:
  - `macro.rs` - Derive macro tests
  - `lattice_properties.rs` - **NEW**: Comprehensive lattice property tests
  - Compile-fail tests for type safety
- **Coverage**: Lattice implementations and properties

#### lattices_macro
- **Location**: `/lattices_macro/src/`
- **Coverage**: Lattice derive macros

### Utility Components

#### variadics
- **Location**: `/variadics/tests/`
- **Tests**:
  - `boxed_refed.rs` - Existing trait tests
  - `variadic_operations.rs` - **NEW**: Comprehensive variadic operations
  - Compile-fail tests
- **Coverage**: Variadic type-level programming

#### variadics_macro
- **Location**: `/variadics_macro/src/`
- **Coverage**: Variadic macro implementations

#### sinktools
- **Location**: `/sinktools/tests/`
- **Coverage**: Comprehensive sink adaptor tests
- **Key Test Files**:
  - `direct.rs` - Direct constructor tests for all adaptors
  - `build.rs` - Builder pattern tests

#### hydro_build_utils
- **Location**: `/hydro_build_utils/tests/`
- **NEW**: Tests added for:
  - `macro_tests.rs` - Build utility macro tests
- **Coverage**: Build-time utilities and macros

#### multiplatform_test
- **Location**: `/multiplatform_test/tests/`
- **Coverage**: Cross-platform testing infrastructure

### Workspace-Level Tests

#### Integration Tests
- **Location**: `/tests/`
- **NEW**: `workspace_integration.rs` - Workspace-level integration tests
- **Coverage**: Cross-crate interactions and workspace configuration

## Running Tests

### Run All Tests
```bash
cargo test --workspace
```

### Run Tests for Specific Crate
```bash
cargo test -p <crate_name>
```

Examples:
```bash
cargo test -p hydro_std
cargo test -p lattices
cargo test -p variadics
cargo test -p sinktools
```

### Run Specific Test
```bash
cargo test --package <crate_name> --test <test_file> <test_name>
```

### Run Tests with Features
```bash
cargo test --workspace --all-features
cargo test -p hydro_lang --features "deploy,sim"
```

### Run Compile-Fail Tests
Compile-fail tests use the `trybuild` crate and run automatically with `cargo test`.

### Run Tests with Different Rust Versions
The project supports both stable and nightly Rust:

```bash
# Stable
cargo +stable test

# Nightly (some features require nightly)
cargo +nightly test
```

## Test Configuration

### Snapshot Tests
Many tests use `insta` for snapshot testing. The `hydro_build_utils` crate provides utilities for managing nightly vs. stable snapshots:

- Stable snapshots: `tests/snapshots/`
- Nightly snapshots: `tests/snapshots-nightly/`

To update snapshots:
```bash
cargo insta test --review
```

### Nightly-Specific Tests
Some crates have nightly-specific features. These are controlled via the `nightly` cfg:

```rust
#[cfg(nightly)]
#[test]
fn test_nightly_feature() {
    // Nightly-only test
}
```

### Platform-Specific Tests
The `multiplatform_test` crate provides the `#[multiplatform_test]` attribute for writing tests that work across different platforms including WASM.

## Test Categories

### Property-Based Tests
Lattice tests verify mathematical properties:
- Commutativity: `a ⊔ b = b ⊔ a`
- Associativity: `(a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)`
- Idempotence: `a ⊔ a = a`
- Identity: `a ⊔ ⊥ = a`

### Behavioral Tests
Most tests verify expected behavior:
- Input/output correctness
- State transitions
- Error handling
- Edge cases

### Conceptual/Documentation Tests
Some new tests document expected behavior without requiring runtime execution:
- These serve as living documentation
- They verify design intent
- They guide future implementation

## Continuous Integration

Tests run automatically on:
- Pull requests
- Pushes to main branch
- Scheduled runs

See `.github/workflows/` for CI configuration.

## Writing New Tests

### Guidelines

1. **Location**: Place tests in the appropriate `tests/` directory or inline with `#[cfg(test)]`

2. **Naming**: Use descriptive names that indicate what is being tested
   ```rust
   #[test]
   fn test_quorum_reaches_minimum_with_successful_responses() { ... }
   ```

3. **Structure**: Organize tests into modules by functionality
   ```rust
   #[cfg(test)]
   mod quorum_basic_tests { ... }
   
   #[cfg(test)]
   mod quorum_edge_cases { ... }
   ```

4. **Documentation**: Include doc comments explaining what the test verifies
   ```rust
   /// Test that quorum collection waits for minimum successful responses
   /// before emitting results.
   #[test]
   fn test_quorum_minimum() { ... }
   ```

5. **Assertions**: Use descriptive assertion messages
   ```rust
   assert_eq!(result, expected, "Quorum should emit after reaching minimum");
   ```

### Test Templates

#### Unit Test
```rust
#[test]
fn test_feature_name() {
    // Arrange
    let input = create_test_input();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected_output);
}
```

#### Property Test
```rust
#[test]
fn test_property_name() {
    let a = create_value();
    let b = create_value();
    
    // Test commutativity
    let mut left = a.clone();
    left.merge(b.clone());
    
    let mut right = b.clone();
    right.merge(a.clone());
    
    assert_eq!(left, right, "Operation should be commutative");
}
```

#### Integration Test
```rust
#[test]
fn test_integration_scenario() {
    // Set up multiple components
    let component_a = setup_component_a();
    let component_b = setup_component_b();
    
    // Test interaction
    let result = component_a.interact_with(component_b);
    
    // Verify end-to-end behavior
    assert!(result.is_ok());
}
```

## Test Utilities

### Test Helpers
Many crates provide test utilities:
- `hydro_test` - Hydro-specific test infrastructure
- `multiplatform_test` - Cross-platform testing
- `example_test` - Example testing framework

### Mock Data
Create reusable test fixtures:
```rust
mod test_fixtures {
    pub fn create_test_data() -> TestData {
        // Reusable test data
    }
}
```

## Troubleshooting

### Test Failures
1. Check error messages carefully
2. Run with `--nocapture` to see println output: `cargo test -- --nocapture`
3. Run single test for isolation: `cargo test specific_test_name`

### Snapshot Mismatches
1. Review changes: `cargo insta review`
2. Accept if correct: `cargo insta accept`
3. Reject if incorrect and fix code

### Nightly/Stable Differences
1. Some tests require nightly features
2. Check `rust-toolchain.toml` for version requirements
3. Use `cargo +nightly test` for nightly-specific tests

## Coverage

To generate coverage reports:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html
```

## Best Practices

1. **Test First**: Consider writing tests before implementation (TDD)
2. **Test Edge Cases**: Don't just test the happy path
3. **Test Errors**: Verify error handling works correctly
4. **Keep Tests Fast**: Unit tests should be quick
5. **Avoid Test Interdependence**: Each test should be independent
6. **Use Clear Names**: Test names should describe what they test
7. **Document Complex Tests**: Add comments for non-obvious test logic
8. **Test Public APIs**: Focus on testing public interfaces
9. **Mock External Dependencies**: Use mocks/stubs for external services
10. **Review Test Output**: Failing tests should provide clear diagnostics

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Book - Tests](https://doc.rust-lang.org/cargo/guide/tests.html)
- [Insta Documentation](https://insta.rs/)
- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)

## Contributing

When adding new features:
1. Add corresponding tests
2. Ensure all existing tests pass
3. Update this document if adding new test categories
4. Follow the project's testing conventions

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.
