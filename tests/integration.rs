//! Integration testing patterns and infrastructure for Lithos.
//!
//! This module establishes integration testing patterns for:
//! - Cross-module API contract testing
//! - Database state management in integration tests
//! - External service mocking for isolated testing
//! - Integration test data fixtures and setup
//!
//! # Note on Integration Test Organization
//!
//! Integration tests in Lithos are organized in two locations:
//! - `tests/integration/` - Top-level integration test infrastructure (this module)
//! - `crates/app/tests/` - Crate-specific integration tests
//!
//! This dual organization allows:
//! - Shared fixtures and utilities in `tests/integration/common.rs`
//! - Crate-specific integration tests that test public APIs
//!
//! Actual integration tests are located in `crates/app/tests/integration_tests.rs`.

// # LINT_DISABLE_REASON: Integration tests do not require public documentation
// | Options tried: Adding docs to every test function
// | Justification: Tests are self-documenting by their names and logic; mandatory docs add noise without value.
#![allow(
    missing_docs,
    reason = "Integration tests do not require public documentation"
)]

pub mod common;
