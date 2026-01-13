# Story: Test Utils Architectural Improvements

Status: in-progress

## Story
As a senior Rust developer working on the Lithos project,
I want to improve the `test-utils` crate with advanced testing patterns,
So that the codebase remains robust, tests are parallelizable, and domain purity is enforced.

## Acceptance Criteria

### 1. Insta Regex Redactions
- **Given** the `with_standard_redactions` function in `insta_utils.rs`
- **When** snapshots contain dynamic values like UUIDs or Timestamps in nested structures
- **Then** these values are automatically redacted using regex-based filters to ensure stable snapshots.
- **Verification**: Fix `SelectorParseError` and demonstrate successful redaction of a complex nested structure.

### 2. Mockall Adoption
- **Given** handwritten `MockRepository` and `StubQueryStore` in `src/cqrs.rs`
- **When** implementing command or query handlers
- **Then** `mockall` generated mocks are used instead of manual stubs.
- **Then** a "Test Adapter" is provided for easy instantiation.

### 3. Parallelism (Context Factories)
- **Given** `shared_mutex` style helpers in `async_utils.rs`
- **When** running multiple tests that interact with the filesystem or database
- **Then** an `IsolatedTestContext` is provided that gives each test a unique temp directory and database namespace.
- **Then** tests can run in parallel without contention.

### 4. Doc-Test Decay
- **Given** `ignore` doc-tests in `lib.rs` and other modules
- **When** running `cargo test --doc`
- **Then** these tests are runnable and verify the `test-utils` API.

### 5. Domain Purity Guardian
- **Given** the `lithos-domain` crate
- **When** checking dependencies in the `test` target
- **Then** an architecture test helper identifies and fails if prohibited crates (like `tokio-fs` or `redb`) are used.

## Tasks / Subtasks

- [ ] **Task 1: Fix Insta Regex Redactions**
    - [ ] Investigate correct `insta` selector and filter syntax for regex redactions.
    - [ ] Update `with_standard_redactions` in `insta_utils.rs` to use regex filters for UUIDs and Timestamps.
    - [ ] Add a test case with nested structures to verify redaction.
- [ ] **Task 2: Adopt Mockall for CQRS**
    - [ ] Add `mockall` dependency to `Cargo.toml`.
    - [ ] Replace manual `MockRepository` with `mockall` mock.
    - [ ] Replace manual `StubQueryStore` with `mockall` mock.
    - [ ] Provide a standard "Test Adapter" for these mocks.
- [ ] **Task 3: Implement Context Factory for Parallelism**
    - [ ] Define `IsolatedTestContext` struct in `async_utils.rs`.
    - [ ] Implement `TestContextFactory` to generate unique contexts.
    - [ ] Update tests to use `IsolatedTestContext`.
- [ ] **Task 4: Audit and Fix Doc-Tests**
    - [ ] Identify all `ignore` doc-tests in `crates/test-utils`.
    - [ ] Convert them to runnable tests by adding necessary imports and setup.
    - [ ] Verify they pass with `cargo test --doc -p test-utils`.
- [ ] **Task 5: Implement Domain Purity Guardian**
    - [ ] Create `arch.rs` in `crates/test-utils/src/`.
    - [ ] Implement a helper that inspects `Cargo.toml` or uses `cargo metadata` to check dependencies of a specific crate's test target.
    - [ ] Add an example test in `lithos-domain` using this helper.

## Quality Assurance
- [ ] Run `mise run verify` to ensure all tests pass.
- [ ] Verify 100% success on all 5 architectural improvements.
