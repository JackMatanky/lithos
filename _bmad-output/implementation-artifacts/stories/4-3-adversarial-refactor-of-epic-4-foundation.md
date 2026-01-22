# Story 4.3: Adversarial Refactor of Epic 4 Foundation

Status: ready-for-dev

<!-- Note: Validation completed. Quality standards enforced. -->

## Story

As an adversarial senior developer,
I want to brutally review and refactor the Epic 4 loading foundation,
So that it follows the leanest, most performant idiomatic Rust practices, balances OOP/FP principles, and eliminates technical debt before testing and documentation.

## Acceptance Criteria

1. **Given** the Epic 4 implementation is complete
   **When** I conduct an adversarial refactor
   **Then** the code follows strict SRP (Single Responsibility Principle) and has zero redundant logic

2. **Given** memory usage concerns
   **When** I optimize the code
   **Then** all `clone()` operations are justified or removed in favor of zero-copy patterns where possible

3. **Given** data structure choices
   **When** I review the implementation
   **Then** enum memory layouts are verified (using `std::mem::size_of`) to ensure compact representation

4. **Given** async I/O requirements
   **When** I audit the filesystem operations
   **Then** `tokio::fs` is used exclusively and NO blocking `std::fs` calls exist

## Tasks / Subtasks (Adversarial Refactor Protocol)

### Task 1: Parser Strategy Audit & Refactor
- [ ] Review `crates/adapters/src/spi/fs/parsers.rs` for SRP violations.
- [ ] Identify and eliminate any redundant logic in format detection or parsing.
- [ ] Analyze `clone()` usage in parser adapters. Refactor to use `&str`, `Cow<str>`, or byte slices where possible (Zero-Copy).
- [ ] Verify `ParseError` enum size. If > 2 words (16 bytes on 64-bit), box large variants to keep the `Result` type small.
- [ ] **Validation:** Ensure all existing parser tests pass.

### Task 2: Path Validation Audit & Refactor
- [ ] Review `crates/adapters/src/spi/fs/validator.rs` for strict SRP.
- [ ] Optimize `validate_relative_path` and `validate_restricted_components` to avoid unnecessary allocation (work with `&Path` and `Components` iterators).
- [ ] Check `resolve_safe_symlink` for efficient path handling (avoiding intermediate `String` allocations).
- [ ] Verify `PathValidationError` enum size. Optimize layout if necessary.
- [ ] **Validation:** Ensure all existing validator tests pass.

### Task 3: Pedantic Linting & Cleanup
- [ ] Run clippy in pedantic mode on `crates/adapters` to identify subtle issues: `cargo clippy -p adapters -- -W clippy::pedantic`.
- [ ] Address valuable pedantic lints (e.g., `clippy::needless_pass_by_value`, `clippy::trivially_copy_pass_by_ref`).
- [ ] **Constraint:** Do not blindly apply all pedantic lints; apply judgment based on readability vs performance.
- [ ] Ensure `#[deny(unsafe_code)]` is strictly respected (no unsafe optimizations).

### Task 4: Async I/O Verification
- [ ] **Audit:** Grep for `std::fs` and ensure NO blocking I/O calls exist in async paths.
- [ ] **Refactor:** Replace any found blocking calls with `tokio::fs` equivalents.
- [ ] **Constraint:** Do NOT add `miette` integration (belongs in CLI layer).

### Task 5: Final Verification
- [ ] Run full test suite: `mise run test`.
- [ ] Verify no regressions in behavior.
- [ ] Confirm code size/complexity reduction (qualitative or quantitative).

## Dev Notes

### Developer Context
This is a **Quality Gate** story. We are pausing to "harden" the implementation of Stories 4.1 and 4.2 before proceeding.
The mindset is **Adversarial**: Assume the previous developer (you) was lazy or missed optimizations.
Focus on **Zero-Cost Abstractions** and **Memory Layout**. Rust enums can become large quickly if not managed, bloating `Result<T, E>` and hurting performance.

### Architectural Compliance
- **SRP:** `parsers.rs` should ONLY parse. `validator.rs` should ONLY validate paths.
- **Zero-Copy:** Favor `&Path`, `&str`, `&[u8]` over `PathBuf`, `String`, `Vec<u8>` in internal APIs.
- **Error Handling:** Keep error types small (`std::mem::size_of::<Error>() <= 16` bytes is a good target). Use `Box<...>` for rare, large error context.

### References
- Epic 4: `_bmad-output/planning-artifacts/epics/epic-4-file-loading-strategy-foundation-mvp-core.md`
- Project Context: `_bmad-output/project-context.md` (Memory Strategy, Error Standards)
- Rust Performance Book (Reference for layout optimizations)

## Dev Agent Record

### Agent Model Used
N/A (Created by SM Agent)

### Debug Log References
- N/A

### Completion Notes List
- N/A

### File List
- `crates/adapters/src/spi/fs/parsers.rs`
- `crates/adapters/src/spi/fs/validator.rs`
