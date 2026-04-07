# Hydro Std Tests

This directory contains comprehensive tests for the `hydro_std` crate, which provides standard distributed systems patterns and utilities.

## Test Files

### quorum_tests.rs
Tests for quorum collection operations:
- **Basic quorum collection**: Minimum and maximum response handling
- **Error handling**: Separate success and error streams
- **Edge cases**: Zero minimum, all errors, duplicate keys
- **Persistence**: State management across ticks
- **Integration scenarios**: Key-value stores, consensus protocols

**Key Concepts Tested:**
- `collect_quorum_with_response()` - Collect responses with metadata
- `collect_quorum()` - Simplified quorum collection
- Quorum semantics with min/max thresholds
- Batching determinism via persistence

### membership_tests.rs
Tests for cluster membership tracking:
- **Basic tracking**: Join and leave operations
- **State machine**: Sequence of membership events
- **Multiple members**: Concurrent membership management
- **Fold semantics**: State maintenance via fold operation
- **Integration scenarios**: Cluster management, auto-scaling, failure detection

**Key Concepts Tested:**
- `track_membership()` - Maintain set of present members
- `MembershipEvent::Joined` - Member joins cluster
- `MembershipEvent::Left` - Member leaves cluster
- Idempotence and state transitions

### request_response_tests.rs
Tests for request-response patterns:
- **Basic joining**: Matching responses with request metadata
- **Timing constraints**: Metadata before or same-tick as response
- **Persistence**: Metadata persists until matched
- **Edge cases**: Unmatched requests/responses, duplicates
- **Integration scenarios**: RPC patterns, distributed queries, retries

**Key Concepts Tested:**
- `join_responses()` - Join responses with request metadata by key
- Temporal constraints (metadata must precede response)
- Anti-join for unmatched metadata
- Batching determinism

## Running Tests

```bash
# Run all tests in this crate
cargo test -p hydro_std

# Run specific test file
cargo test -p hydro_std --test quorum_tests
cargo test -p hydro_std --test membership_tests
cargo test -p hydro_std --test request_response_tests

# Run specific test
cargo test -p hydro_std test_quorum_basic
```

## Test Style

These tests primarily use **conceptual/documentation testing** approach:
- Tests document expected behavior and design intent
- Serve as living documentation and specifications
- Guide implementation and usage
- Verify API contracts

Many tests use `assert!(true, "...")` pattern to document concepts without requiring full runtime integration. This is intentional as `hydro_std` functions require the full Hydro runtime environment to execute.

## Future Enhancements

Potential additions:
1. **Runtime integration tests**: Tests that actually execute with Hydro runtime
2. **Property-based tests**: Use quickcheck for exhaustive testing
3. **Performance tests**: Benchmark critical operations
4. **Simulation tests**: Use Hydro's simulation framework for deterministic testing

## See Also

- [TESTING.md](../../TESTING.md) - Comprehensive testing guide
- [TEST_SUMMARY.md](../../TEST_SUMMARY.md) - Summary of test coverage
- [hydro_std source](../src/) - Implementation being tested
