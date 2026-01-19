# Story 4.5: Adversarial Refactor of Epic 4 Foundation

Status: pending

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As an adversarial senior developer,
I want to brutally review and refactor the Epic 4 loading foundation,
So that it follows the leanest, most performant idiomatic Rust practices, balances OOP/FP principles, and eliminates technical debt before testing and documentation.

## Acceptance Criteria

1. **Given** the Epic 4 implementation is complete
   **When** I conduct an adversarial refactor
   **Then** the code follows strict SRP (Single Responsibility Principle) and has zero redundant logic

2. **Given** Rust-specific performance targets
   **When** I optimize the code
   **Then** all expensive operations (clones, allocations) are justified or eliminated in favor of zero-copy patterns where possible

3. **Given** the hybrid OOP/FP architecture
   **When** I review the implementation
   **Then** I ensure appropriate use of traits (OOP) vs functional patterns (iterators, closures, immutable state)

4. **Given** the hexagonal architecture
   **When** I check boundaries
   **Then** there is zero leakage of infrastructure details into the domain layer

5. **Given** the refactor is complete
   **When** I run `mise run verify`
   **Then** all quality gates pass with zero warnings and cognitive complexity remains < 25

## Tasks / Subtasks

### Task 1: Architectural Integrity & Boundary Audit
- [ ] Review all Epic 4 interfaces and ensure proper trait-based abstraction (Hexagonal Ports)
- [ ] Verify that the `domain` layer has ZERO knowledge of specific parsing libraries (serde_json, toml, etc.)
- [ ] Ensure that format-specific logic is strictly contained within the `adapters` layer
- [ ] Audit the `UnitOfWork` and `TransactionContext` usage for any transactional loading scenarios

### Task 2: Performance & Memory Optimization
- [ ] Audit all uses of `.clone()` and `.to_owned()`; replace with references or zero-copy `Cow` where appropriate
- [ ] Optimize the format detection logic to avoid multiple re-reads of the same file buffer
- [ ] Ensure `tokio::task::spawn_blocking` is used correctly for all synchronous file I/O operations
- [ ] Validate that large file buffers are handled efficiently (e.g., using streams or buffered readers)

### Task 3: Idiomatic Rust & Clean Code Refactor
- [ ] Refactor complex `match` or `if let` blocks into clean, functional chains using `Option` and `Result` combinators
- [ ] Ensure error handling uses `thiserror` and `anyhow` correctly with high-fidelity context
- [ ] Replace imperative loops with idiomatic iterators (`map`, `filter`, `fold`) where it improves clarity
- [ ] Audit variable naming for strict compliance with the project's semantic naming rules

### Task 4: OOP vs FP Balance Review
- [ ] Verify that state is minimized and immutable by default
- [ ] Ensure traits are used for behavior abstraction, not just as "interfaces" for the sake of it
- [ ] Review the use of generics vs dynamic dispatch (`Arc<dyn Trait>`) for optimal performance/flexibility balance

### Task 5: Quality Assurance & Verification
- [ ] Run `mise run fmt` to ensure perfect alignment with project style
- [ ] Run `mise run lint` (Clippy) and address EVERY warning, even if allowed elsewhere
- [ ] Run `mise run verify` to confirm all existing tests still pass after refactoring
- [ ] Run `pre-commit run --all-files` to ensure all quality hooks pass

## Dev Notes

### Adversarial Focus
- **The "Brutal" Lens**: Don't just fix bugs; fix "smells". If a function is 50 lines but could be 10 with a better iterator chain, refactor it.
- **Zero-Copy Priority**: Since this is a core loading foundation, performance at the base level is critical for the rest of the application.
- **Error Fidelity**: Ensure error messages are not just "failed to load" but provide miette-quality diagnostics.

### Technical Standards
- **Cognitive Complexity**: Max 25 (deny)
- **Function Length**: Max 100 lines (deny)
- **Rust Edition**: 2024 (1.92+)
