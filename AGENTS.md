# Lithos Rust - AI Agent Reference

## Critical Files - READ FIRST

**MUST** review these files before starting any work:

- **Project Context**: [Core rules and patterns](_bmad-output/project-context.md)
- **PRD**: [Product requirements](_bmad-output/planning-artifacts/prd.md)
- **Architecture**:
  - [Core Architectural Decisions](_bmad-output/planning-artifacts/architecture/core-architectural-decisions.md)
  - [Implementation Patterns & Consistency Rules](_bmad-output/planning-artifacts/architecture/implementation-patterns-consistency-rules.md)
  - [Project Structure & Boundaries](_bmad-output/planning-artifacts/architecture/project-structure-boundaries.md)

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
- **Architecture**: Port-Based CQRS with Bounded Contexts - business contexts isolated, generic CQRS over storage ports
- **Key Libraries**: redb (zero-copy DB), rkyv (serialization), pulldown-cmark (markdown parsing)
- **Testing**: nextest, criterion benchmarks, tarpaulin coverage
- **Build**: cargo workspace with mise task orchestration

### Critical Coding Standards
- **Zero-copy patterns** for performance-critical paths via GAT-based port traits
- **Port-based CQRS**: CQRS types generic over split storage ports (`Query<Q: SchemaQueryPort>`, `Command<C: SchemaCommandPort>`)
- **Context isolation**: Business contexts (note, schema, template) don't import each other
- **Type-driven design**: Private fields by default, validation at construction, newtype wrappers
- **Test-first development**: Red-green-refactor cycle required
- **ADR documentation**: All architectural decisions documented in [docs/adr/](docs/adr/)

### Project Structure
- `lithos-core/` - Core library with all business logic and infrastructure
  - `src/config/` - Configuration management context
  - `src/note/` - Note domain context
  - `src/schema/` - Schema domain context
  - `src/template/` - Template domain context
  - `src/db/` - Database infrastructure
  - `src/fs/` - Filesystem utilities
  - `benches/` - Performance benchmarks
- `lithos-cli/` - Command-line interface binary

For complete rules, see [_bmad-output/project-context.md](_bmad-output/project-context.md)

## Key Architectural Constraints

⚠️ **NON-NEGOTIABLE RULES**:
1. **Context isolation**: Business contexts (note, schema, template) MUST NOT import each other
   - Config is cross-cutting infrastructure (available to all contexts)
   - Only infrastructure (db, fs, patterns) and config may be imported by business contexts
2. **Port-based CQRS**: CQRS types MUST be generic over split storage port traits
   - `Query<Q: SchemaQueryPort>`, `Command<C: SchemaCommandPort>`
   - Ports split into Query and Command to prevent interface bloat
   - Storage ports use GATs for zero-copy: `type Archived<'a> where Self: 'a`
3. **Type safety**: Private fields by default, validation at construction, newtype wrappers for domain constraints
4. **Zero-copy patterns**: Domain types have rkyv derives; use `Stored*` types only when domain shape is inefficient for storage
5. **Test-first**: Red-green-refactor cycle required - tests before implementation
6. **ADRs required**: Document all architectural decisions in [docs/adr/](docs/adr/)
7. **Dependency flow**: Infrastructure (db, fs, config, patterns) → Business Contexts (note, schema, template) → CLI

## Where Does This Code Go?

**Pure business logic (no I/O)?** → `lithos-core/src/{context}/` (note, schema, template)
**Cross-cutting configuration?** → `lithos-core/src/config/`
**Database operations?** → `lithos-core/src/db/`
**File system utilities?** → `lithos-core/src/fs/`
**CLI interface?** → `lithos-cli/src/`
**Tests for domain logic?** → Same file as impl with `#[cfg(test)]`
**Integration tests?** → `lithos-core/tests/`
**Benchmarks?** → `lithos-core/benches/`

## Critical Rust Patterns & Anti-Patterns

For deeper guidance on Rust style/module organization/tooling and crate-specific usage, start at [docs/refs/rust/README.md](docs/refs/rust/README.md).

### Repo-Specific Rust Notes (High Signal)

- **Clippy suppressions**: Prefer local `#[expect(clippy::lint_name, reason = "...")]` over `#[allow(...)]`; avoid crate/module-wide suppressions unless it’s a deliberate policy.
- **Doc tests**: `nextest` does not run doctests; when changing public docs/examples, also run `cargo test --doc`.
- **Module layout**: Contexts use `<context>/mod.rs` pattern with submodules for organization.
- **Rustdoc hygiene**: For fallible/unsafe/panicking public APIs, document `# Errors`, `# Safety`, and/or `# Panics` (see [docs/refs/rust/style.md](docs/refs/rust/style.md)).

### Zero-Copy Library Footguns (Read Before Editing Hot Paths)

- **rkyv format control**: Treat endianness/alignment/pointer-width feature choices as a persisted-format contract; changing them is a breaking change for on-disk bytes (see [docs/refs/crates/rkyv.md](docs/refs/crates/rkyv.md)).
- **rkyv validation**: Use `rkyv::access` at trust boundaries (files/network/user input); reserve `access_unchecked` for trusted, internally produced bytes.
- **redb guards**: `AccessGuard` values borrow the transaction/table; do not return or store them beyond the transaction scope.
- **redb custom Value**: Due to orphan rules, implement `redb::Value` via local newtypes/wrappers when you need custom encoding.
- **moka determinism**: Cache stats are eventually consistent; in tests that assert stats/entry counts, call `run_pending_tasks()`.
- **moka callbacks**: Eviction listeners must not panic and should be fast (they run on user threads).

## Rust Idioms (Rules)

These rules operationalize common Rust idioms for day-to-day Lithos development.
For deeper rationale and examples, see [docs/refs/rust/idioms.md](docs/refs/rust/idioms.md).

### API & Ownership
- Prefer borrowed arguments in APIs: take `&str`, `&Path`, slices, and `&T` (or `impl AsRef<Path>` / `impl Borrow<T>`) instead of `String`/`PathBuf`/owned types unless ownership is required.
- Use `impl Trait`/generics for “accept anything that can be viewed as X” APIs; reserve `&dyn Trait` for intentional runtime polymorphism.
- When ownership is required, make it explicit: take `T`/`Box<T>`/`Arc<T>` by value and document the transfer.

### Construction & Defaults
- Use conventional constructors: `new()` for infallible, `try_new()` / `new_checked()` for fallible, and `from_*`/`try_from_*` conversions via `From`/`TryFrom`.
- Prefer builders when there are many optional parameters or invariants to enforce; keep `new()` small and unsurprising.
- Implement or derive `Default` when a sensible default exists; prefer struct update syntax (`..Default::default()`) for ergonomic initialization.

### Strings & Formatting
- Use `format!`/`write!`/`writeln!` for structured string construction; avoid repeated `+` concatenation in loops.
- Accept string inputs as `&str` (or `impl AsRef<str>` when appropriate); store immutable string data as `Box<str>` when ownership is needed and mutability isn’t.

### Mutation, Moves, and Invariants
- Keep `mut` scopes tight: prefer temporary mutability (shadowing) to long-lived `mut` bindings.
- When you need to move out of a field or replace a value, prefer `std::mem::take` / `std::mem::replace` over cloning.
- Prefer iterators over indexing; when indexing is unavoidable, use `.get()` and handle `None`.
- Treat `Option` as an iterable for control flow: use `if let`, `while let`, `.into_iter()`, and combinators (`map`, `and_then`, `ok_or`) instead of sentinel values.

### Resource Management
- Use RAII: acquire resources in constructors and release in `Drop`; avoid “manual close” APIs unless required for performance or correctness.
- Never panic across FFI boundaries; Rust must not unwind into C.

### Closures & Captures
- Be explicit about closure capture semantics: use `move` when the closure must own captured values.
- When a closure needs owned data but the surrounding scope still needs it, explicitly rebind (e.g., clone an `Arc`/`String` into a new binding) rather than fighting the borrow checker.

### Extensibility & Public API Evolution
- For public enums/structs intended to evolve, use `#[non_exhaustive]` (or private fields) to prevent downstream exhaustive construction/matching.
- When matching on non-exhaustive enums, always include a wildcard arm to preserve forward compatibility.

### Documentation & Doctests
- Write rustdoc examples as compilable code; hide setup noise in doctests using `#` lines to keep examples readable.

### Error Handling & FFI Interop
- Prefer `Result<T, E>` with structured errors (`thiserror` in non-domain crates); avoid `unwrap()`/`expect()` in production.
- For fallible operations that consume an input, prefer returning the consumed value on failure (e.g., `Result<T, (E, Input)>` or an error type that carries the input) when it materially improves recovery.
- In FFI:
	- Accept strings as `*const c_char` + `CStr`; pass strings as `CString`/`*const c_char` with clear ownership rules.
	- Return errors as status codes and/or out-parameters; ensure all FFI-exposed functions are `extern "C"` and panic-free.

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
- [ ] Context boundaries respected (business contexts isolated, no cross-imports)
- [ ] Port-based CQRS pattern followed (generic over storage traits)
- [ ] Type-driven design applied (private fields, validated constructors)
- [ ] Documentation updated (doc comments for public APIs)
- [ ] Doc tests run when docs/examples changed (`cargo test --doc`)
- [ ] ADR created if architectural decision made

## Before Submitting Work

1. **Run full verification**: `mise run verify` must be 100% green
2. **Review test quality**: Critical paths tested, edge cases covered
3. **Code hygiene check**: No debug prints, commented code, or TODOs
4. **Documentation**: If architectural change, ADR created in `docs/adr/`

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
| `mise run test`              | Run all tests (unit, integration, e2e) (alias: `t`).                              |
| `mise run test:unit`         | Run all unit tests using `nextest` (alias: `tu`).                                 |
| `mise run test:unit:core`    | Run core crate unit tests (alias: `tucore`).                                      |
| `mise run test:unit:cli`     | Run CLI crate unit tests (alias: `tucli`).                                        |
| `mise run test:unit:config`  | Run config module unit tests (alias: `tuconf`).                                   |
| `mise run test:unit:note`    | Run note module unit tests (alias: `tunote`).                                     |
| `mise run test:unit:schema`  | Run schema module unit tests (alias: `tusch`).                                    |
| `mise run test:unit:template`| Run template module unit tests (alias: `tutemp`).                                 |
| `mise run test:unit:db`      | Run db module unit tests (alias: `tudb`).                                         |
| `mise run test:unit:fs`      | Run fs module unit tests (alias: `tufs`).                                         |
| `mise run test:bench`        | Run all performance benchmarks using `criterion`.                                 |
| `mise run test:bench:core`   | Run core crate benchmarks (alias: `tbcore`).                                      |
| `mise run test:bench:cli`    | Run CLI crate benchmarks (alias: `tbcli`).                                        |
| `mise run test:integration`  | Run all integration tests across the workspace (alias: `ti`).                     |
| `mise run test:e2e`          | Run end-to-end tests (alias: `te`).                                               |
| `mise run test:coverage`     | Generate code coverage reports using `tarpaulin` (alias: `tc`).                   |
| `mise run test:watch`        | Watch mode: automatically run tests on file changes (alias: `tw`).                |
| `mise run test:burn-in`      | Run tests repeatedly to detect flaky failures (alias: `tb`).                      |
| `mise run test:changed`      | Run tests only for crates affected by changes (alias: `tc`).                      |
