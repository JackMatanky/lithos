# Lithos Rust - AI Agent Reference

## Critical Files - READ FIRST

**MUST** review these files before starting any work:

- **Project Context**: [_bmad-output/project-context.md](_bmad-output/project-context.md) - Core rules and patterns
- **Workflow Status**: [_bmad-output/planning-artifacts/bmm-workflow-status.yaml](_bmad-output/planning-artifacts/bmm-workflow-status.yaml) - Current phase and next steps
- **Architecture**: [_bmad-output/planning-artifacts/architecture/](_bmad-output/planning-artifacts/architecture/) - System design decisions
- **PRD**: [_bmad-output/planning-artifacts/prd.md](_bmad-output/planning-artifacts/prd.md) - Product requirements

## BMAD Agent Activation

To activate specialized agents, use: `"As [agent-name], ..."` (e.g., `"As dev, implement the cache service"`)

**Available agents**: See [_bmad/_config/agent-manifest.csv](_bmad/_config/agent-manifest.csv) for full list
- **dev** - Implementation, debugging, refactoring
- **architect** - System design, ADRs, tech selection
- **tea** - Test strategy, quality gates
- **quick-flow-solo-dev** - Rapid prototyping
- **bmad-master** - General orchestration

**Available workflows**: See [_bmad/_config/workflow-manifest.csv](_bmad/_config/workflow-manifest.csv)

## Project-Specific Context

### Technology Stack
- **Language**: Rust (latest stable)
- **Architecture**: Hexagonal (Ports & Adapters) - domain isolated from infrastructure
- **Key Libraries**: redb (zero-copy DB), moka (concurrent cache), rkyv (serialization)
- **Testing**: nextest, criterion benchmarks, tarpaulin coverage
- **Build**: cargo workspace with mise task orchestration

### Critical Coding Standards
- **Zero-copy patterns** for performance-critical paths (see [Technical Reference](#technical-reference-documentation))
- **Hexagonal architecture**: Domain crate has no external dependencies
- **Test-first development**: Red-green-refactor cycle required
- **ADR documentation**: All architectural decisions documented in [docs/adr/](docs/adr/)

### Project Structure
- `crates/domain/` - Pure business logic (no I/O, no external deps)
- `crates/app/` - Application services and use cases
- `crates/adapters/` - Infrastructure (DB, file system, cache)
- `crates/cli/` - Command-line interface
- `tests/` - Integration and E2E tests
- `benches/` - Performance benchmarks

For complete rules, see [_bmad-output/project-context.md](_bmad-output/project-context.md)

## Key Architectural Constraints

⚠️ **NON-NEGOTIABLE RULES**:
1. **Domain purity**: `crates/domain/` MUST have zero external dependencies
2. **Zero-copy patterns**: Use rkyv for serialization, avoid cloning in hot paths
3. **Test-first**: Red-green-refactor cycle required - tests before implementation
4. **ADRs required**: Document all architectural decisions in [docs/adr/](docs/adr/)
5. **Hexagonal architecture**: Domain → App → Adapters (dependencies flow inward only)

## Where Does This Code Go?

**Pure business logic (no I/O)?** → `crates/domain/src/`
**Application orchestration?** → `crates/app/src/`
**Database/cache/file operations?** → `crates/adapters/src/spi/`
**API/CLI interface?** → `crates/adapters/src/api/` or `crates/cli/src/`
**Tests for domain logic?** → Same file as impl with `#[cfg(test)]`
**Integration tests?** → `tests/suite/integration/`
**Benchmarks?** → `benches/`

## Critical Rust Patterns & Anti-Patterns

### ✅ Always Do
- **Error handling**: Use `Result<T, E>` with `?` operator, never `unwrap()`/`expect()` in production
- **Paths**: Use `PathBuf` (owned) or `&Path` (borrowed), NEVER `String` for file paths
- **String efficiency**: Use `&str` for borrows, `Box<str>` for immutable data, `String` only when mutable
- **Async blocking**: Use `tokio::task::spawn_blocking` for any `std::fs` or CPU-intensive work
- **Collections**: Use `.get()` instead of `[index]`, `entry()` API for HashMap updates
- **Conversion traits**: Implement `From/Into` for infallible conversions, `TryFrom/TryInto` for fallible ones
- **Lifetimes as documentation**: `fn get<'a>(&'a self) -> Guard<'a>` shows zero-copy, `fn get(&self) -> T` hides allocation

### ❌ Never Do
- **String cloning for paths**: Path operations must use `Path`/`PathBuf` APIs
- **Clone in traits**: `trait Cache<V: Clone>` forces all implementations to allocate
- **Unwrap/panic**: Use `?`, `ok_or()`, `context()` - panics crash the process
- **Async mutex across await**: NEVER hold `std::sync::MutexGuard` across `.await` (deadlock risk)
- **Numeric casting with 'as'**: Use `.try_into()?` to catch overflow/truncation errors
- **Generic `String` errors**: Use `thiserror` for structured errors with context
- **Ad-hoc conversions**: Don't write `to_x()` methods - use `From/Into` traits instead

### 🎯 Performance Patterns
- **Zero-copy reads**: Return guards (`Deref<Target=V>`) not owned values
- **Batch transactions**: One redb write transaction for bulk operations, not one per item
- **Arc for shared state**: `Arc<T>` is cheap to clone (atomic refcount), clone the Arc not T
- **Avoid premature async**: moka and redb are sync - adding async adds 50ns overhead

## Definition of Done

Before marking any task complete:
- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] All public APIs have tests (functions, methods, traits)
- [ ] Tests cover critical paths and business logic (not chasing % targets)
- [ ] No `unwrap()`/`panic!` in production code
- [ ] Hexagonal boundaries respected (domain has zero external deps)
- [ ] Documentation updated (doc comments for public APIs)
- [ ] ADR created if architectural decision made

## Before Submitting Work

1. **Run full verification**: `mise run verify` must be 100% green
2. **Check architecture compliance**: `mise run test:arch` passes
3. **Review test quality**: Critical paths tested, edge cases covered
4. **Code hygiene check**: No debug prints, commented code, or TODOs
5. **Documentation**: If architectural change, ADR created in `docs/adr/`

## Common Commands (mise tasks)

| Command                      | Action                                                                            |
| :--------------------------- | :-------------------------------------------------------------------------------- |
| `mise run verify`            | Full quality gate orchestration (fmt + lint + tests + adr:validate) (alias: `v`). |
| `mise run quality`           | Run all quality gates (fmt, lint, adr:validate) (alias: `q`).                     |
| `mise run lint`              | Run linting checks using clippy.                                                  |
| `mise run fmt`               | Format code using rustfmt.                                                        |
| `mise run deny`              | Check dependencies for security and license issues.                               |
| `mise run clean`             | Clean build artifacts and temporary files.                                        |
| `mise run clean:cargo`       | Clean only cargo build artifacts.                                                 |
| `mise run clean:test`        | Clean only test output artifacts.                                                 |
| `mise run clean:reports`     | Clean only coverage and JUnit reports.                                            |
| `mise run build`             | Build the project binaries.                                                       |
| `mise run doc`               | Generate and open project documentation.                                          |
| `mise run dev-setup`         | Set up development environment and dependencies.                                  |
| `mise run adr:validate`      | Validate ADR files for compliance.                                                |
| `mise run adr:metrics`       | Generate metrics for ADR management.                                              |
| `mise run ci`                | Simulate CI/CD pipeline.                                                          |
| `mise run timing`            | Run verify with detailed timing information.                                      |
| `mise run test`              | Run all unit and integration tests (alias: `t`).                                  |
| `mise run test:unit`         | Run all unit tests across the workspace using `nextest`.                          |
| `mise run test:unit:<crate>` | Run unit tests for a specific crate (e.g., `test:unit:app`).                      |
| `mise run test:unit:domain`  | Run domain crate unit tests (alias: `tud`).                                       |
| `mise run test:unit:app`     | Run app crate unit tests (alias: `tuap`).                                         |
| `mise run test:unit:adapters`| Run adapters crate unit tests (alias: `tuad`).                                    |
| `mise run test:unit:cli`     | Run CLI crate unit tests (alias: `tuc`).                                          |
| `mise run test:bench`        | Run all performance benchmarks using `criterion`.                                 |
| `mise run test:bench:domain` | Run domain crate benchmarks (alias: `tbd`).                                       |
| `mise run test:bench:app`    | Run app crate benchmarks (alias: `tbap`).                                         |
| `mise run test:bench:adapters`| Run adapters crate benchmarks (alias: `tbad`).                                   |
| `mise run test:bench:cli`    | Run CLI crate benchmarks (alias: `tbc`).                                          |
| `mise run test:integration`  | Run all integration tests across the workspace.                                   |
| `mise run test:e2e`          | Run end-to-end tests using `cli_smoke` binary.                                    |
| `mise run test:arch`         | Run architectural enforcement tests using `purity` binary.                        |
| `mise run test:coverage`     | Generate code coverage reports using `tarpaulin`.                                 |
| `mise run test:watch`        | Watch mode: automatically run tests on file changes.                              |
