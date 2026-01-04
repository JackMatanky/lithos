---
project_name: 'lithos'
user_name: 'Jack'
date: '2026-01-04T15:15:06Z'
sections_completed: ['technology_stack', 'language_rules', 'framework_rules', 'testing_rules', 'quality_rules', 'workflow_rules', 'anti_patterns']
status: 'complete'
rule_count: 49
optimized_for_llm: true
existing_patterns_found: 5
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

- **Tooling & Automation**
  - `mise` is the primary task runner: use `mise run <task>` for builds, tests, lint, and perf jobs. It auto-installs `go`, `golangci-lint`, and `pre-commit`, pins versions via lockfile, and writes binaries to `./bin`. `justfile` exists for historical compatibility only; prefer mise tasks.
  - `golangci-lint v2`, `pre-commit`, and `gitleaks v8.28.0` run through `mise run format`, `mise run lint`, `mise run verify`, and hook automation.
- **Language & Runtime**
  - `Go 1.24.0` (module directive) — all code must compile against Go 1.24+, and `mise run verify` enforces gofmt, lint, and go test.
- **CLI & Configuration**
  - `github.com/spf13/cobra v1.10.1` — CLI adapter implementing `CLIPort` for commands such as `lithos new`/`index`.
  - `github.com/spf13/viper v1.21.0` — configuration adapter loading `lithos.json`, env vars, and flags via `ConfigPort`.
- **Markdown, Schemas & Templates**
  - `github.com/yuin/goldmark v1.5.4` + `go.abhg.dev/goldmark/frontmatter v0.2.0` — adapters parse markdown, YAML frontmatter, links, headings, tags, and tasks in one AST traversal.
  - `gopkg.in/yaml.v3 v3.0.1` — YAML utilities for schema fixtures and golden tests.
  - `text/template` — TemplateEngine provides prompt/query helpers for vault-aware rendering.
- **Storage & Validation Infrastructure**
  - `go.etcd.io/bbolt v1.4.3` — hot cache (metadata lookups) and MetadataQueryPort indexes.
  - `modernc.org/sqlite v1.39.1` — cold cache for schema-aware SQL queries, synchronized with BoltDB using CacheUnitOfWork.
  - `github.com/moby/sys/atomicwriter v0.1.0` — guarantees atomic writes when persisting vault and cache data.
- **Logging, Terminal UX & Testing**
  - `github.com/rs/zerolog v1.34.0` — structured logging; always inject scoped loggers.
  - `golang.org/x/term v0.25.0` — terminal detection for interactive prompts and finders.
  - `github.com/stretchr/testify v1.11.1` — assertions/mocks for unit tests; coverage, perf, and baseline artifacts must be written to `tests/artifacts/*` (coverage, profiles, reports, baseline), never the repo root.

## Critical Implementation Rules

### Language-Specific Rules

- **Context-first APIs:** Any function that might block, perform I/O, or spawn goroutines must take `ctx context.Context` as the first parameter and propagate it; packages must not call `context.Background()` internally. Cancel contexts in deferred cleanups.
- **Logging discipline:** Accept a scoped `zerolog.Logger` in constructors, derive sub-loggers with `.With().Str("component", ...).Logger()`, and never use `fmt.Print*` or the standard `log` package.
- **Validation naming:** Adapter structs expose `ValidateSyntax()`/`IsValidStructure()` for format checks; domain entities expose `Validate()` for semantic rules. Keep the naming consistent to avoid mixing responsibilities.
- **Error semantics:** Define domain errors (e.g., `NoteValidationError`) with contextual fields, implement `Error()` and `Unwrap()` when wrapping, and always wrap upstream errors using `%w` so callers can use `errors.Is/As`.
- **Defensive copies:** Constructors for aggregates (`Note`, `Frontmatter`, registries) must copy incoming slices/maps and expose read-only views to prevent template code or adapters from mutating shared state.
- **Template helpers:** Register all template funcs via `template.FuncMap` *before* parsing templates. Functions must remain deterministic/pure and avoid filesystem access—side effects belong in domain services.
- **Concurrency etiquette:** When sharing state (registries, caches), guard with `sync.RWMutex` or channel ownership. Any goroutine must respect the parent context, surface errors via structured channels, and document ownership in comments.
- **Go formatting:** All code passes gofmt/golines/gci ordering through `mise run format`; never hand-edit import order or rely on goimports defaults.
- **Generics usage:** Custom types like `Result[T]` prefer `any` constraints only when necessary; avoid type assertions by adding helper methods, and keep exported generic helpers in shared packages.

### Framework-Specific Rules

- **Cobra command layering:** Each `cobra.Command` handles flags/args and calls `CLICommander`/ports. Use `PersistentPreRunE`/`PreRunE` for CLI-only validation (e.g., vault path checks) and keep `RunE` thin wrappers over domain calls.
- **Finder/prompt adapters:** Fuzzy finders and prompts must flow through `FinderPort`/`InteractivePort` so the same logic works for the future BubbleTea adapter—never import UI libraries directly into commands.
- **Viper access boundaries:** Only the Config SPI adapter touches Viper. Downstream services receive `domain.Config` via constructors, and tests create config structs manually without loading real files.
- **TemplateEngine integration:** Template commands resolve templates through `TemplateEngine`, leverage its helpers (prompt/query/path), and never parse or execute `text/template` directly.
- **SPI adapter discipline:** Vault/schema/cache adapters perform all file I/O and syntactic validation before creating domain models. Domain packages never import adapter packages or touch the filesystem.

### Testing Rules

- **Test organization:** Unit tests live beside source packages as table-driven `*_test.go` files; integration/performance suites live under `tests/<type>` and run via `mise run test:int`, `test:security`, `perf:*`, etc. Golden fixtures belong in `testdata/` mirrors.
- **Invocation & outputs:** Always run `mise` tasks (`test`, `test:v`, `test:cov`, `test:artifacts`) so coverage, profiles, and perf results land under `tests/artifacts/(coverage|profiles|reports|baseline)` rather than repo root.
- **Mocks & doubles:** Prefer real structs or lightweight fakes. When mocking ports, use `testify/mock` or in-memory adapters; never rely on globals or package-level state.
- **Build tags:** Integration/security/compliance tests must stay behind build tags and should document the required `go test -tags=...` invocation in the file header.
- **Performance harness:** `mise run perf:bench:*`, `perf:profile:*`, and `perf:baseline` capture metrics/baselines; baselines include timestamps and git commit hashes under `tests/artifacts/baseline`.
- **Coverage requirements:** `mise run test:cov` must create `coverage.out` and HTML reports using `-covermode=atomic`; fail fast if artifacts cannot be produced.
- **Hermetic fixtures:** Use `testing/fstest` and the synthetic vault builders under `tests/utils` to avoid touching real user files. Any test hitting disk must write into `tests/artifacts` and clean up after itself.

### Code Quality & Style Rules

- **Lint/format pipeline:** Run `mise run format` and `mise run lint` (golangci-lint v2) before any commit. The config enforces gofmt, goimports, golines (80 cols), GCI ordering, and the enabled linters—never skip hooks.
- **Import aliases:** Respect `.golangci.toml` `importas` rules (`lithosErr`, `lithosLog`, `templateAdapter`, `vaultAdapter`, etc.). Add new aliases to the config rather than inventing ad-hoc names.
- **Declaration order:** Within a file use `const → var → type → func`, keeping exported symbols first. Unexported helpers and test-only utilities come afterward.
- **File naming:** File names stay short and omit redundancies—avoid repeating parent directories or appending `adapter` just because the file lives under `internal/adapters/...`. Prefer single-word names (e.g., `reader.go` inside the vault adapter folder) while keeping enough context.
- **Adapter type names:** Structs that represent adapters may keep the `Adapter` suffix because they’re imported into other packages where the role isn’t obvious; reserve the suffix for types, not files.
- **Scope-driven identifiers:** Local/private identifiers should be descriptive for clarity, whereas exported or widely referenced identifiers should stay concise for developer ergonomics.
- **Naming conventions:** Interfaces that cross architecture boundaries end in `Port`. Adapters inherit the domain name via their folder path; avoid duplicating path segments in names unless disambiguation is needed.
- **Documentation:** Package doc comments must exist (godoclint). Exported types/functions require full-sentence comments explaining *why* the code exists. Inline comments are for non-obvious logic only.
- **Magic numbers & configuration:** Promote repeated literals to `const` blocks; rely on typed config structs or builder options instead of scattering raw ints.
- **Error logging:** Log errors with `.Err(err)` and `.Msg("context")`; never embed errors in formatted strings or drop them. Keep log keys consistent (`component`, `operation`, `path`, `count`).
- **Folder discipline:** Adapter code stays in `internal/adapters/{api,spi}/...`, application orchestration in `internal/app`, domain entities in `internal/domain`, and shared utilities in `internal/shared` (which never import adapters).

### Development Workflow Rules

- **Task runner:** Always run `mise run verify` (format → lint → test) before pushing. It wires golangci-lint, gofmt, go test, coverage, and keeps tool versions pinned via the mise lockfile.
- **Pre-commit hooks:** Install via `mise run setup:pre-commit` (or `pre-commit install`) and keep them enabled; hooks run formatting checks, config validators, and gitleaks scans on every commit. Hooks must pass—no `nolint`, `--no-verify`, or similar bypasses.
- **Secrets scanning:** `gitleaks` runs both locally and in CI; never commit secrets or disable the hook. Document false positives in `.gitleaks.toml` instead of removing the check.
- **Branch/worktree hygiene:** Develop in Git worktrees or feature branches named after the work item (e.g., `feature/indexer-cache-hit`). Keep worktrees isolated per task and never push directly to `main`/`master`.
- **Commit etiquette:** Use Conventional Commits (e.g., `feat: add cache invalidation`). Group related changes—even if staging happens in multiple steps—and only commit after `mise run verify` succeeds locally.
- **PR checklist:** Before opening a PR, ensure `mise run format`, `mise run lint`, `mise run test`, and `mise run gitleaks` (as needed) are clean. Update docs/tests, describe the intent, and provide reproduction steps or context links.
- **CI monitoring:** After pushing, watch GitHub Actions to ensure all workflows pass; never merge with failing or pending CI.
- **Artifact cleanliness:** Keep build/test outputs under `tests/artifacts/**` (coverage, profiles, reports, baseline). Clean them via `mise run clean` when done; nothing should spill into repo root.

### Critical Don't-Miss Rules

- **Vault safety:** Never mutate files outside the configured vault path or `.lithos/cache/**`. Tests must operate on synthetic vaults; production code must respect the configured `vaultPath`.
- **Adapter bypass:** Do not access the filesystem, config files, or interactive prompts directly from domain/services. All I/O flows through SPI adapters; bypassing them breaks hexagonal boundaries and makes testing impossible.
- **Validation shortcuts:** Never skip adapter-level syntactic validation or domain semantic validation—skipping either corrupts caches or emits malformed notes.
- **Cache integrity:** BoltDB (hot) and SQLite (cold) caches must be updated via `CacheUnitOfWork`. Direct writes to either store risk divergence; always use the provided ports/services.
- **Template side effects:** Template helpers must stay pure (no filesystem/network). Business operations belong in domain services; templates only render data and interact through sanctioned helper functions.
- **Concurrency hazards:** Shared registries, cache writers, and vault scanners require locking or goroutine ownership. Spawning goroutines without context cancellation or error propagation leaks work and leaves indexes inconsistent.
- **Secrets & logs:** Never log vault content, note bodies, or secrets. Logs should only include sanitized paths/IDs when essential.
- **Performance traps:** Avoid re-walking the vault tree or reparsing schemas on each command. Use VaultIndexer and SchemaEngine, which cache results; ad-hoc scans blow past performance targets.
- **Command wiring discipline:** `cmd/lithos/main.go` should stay a DI harness—even if it grows verbose—so every dependency is wired explicitly via constructors. That verbosity is preferable to creating god-object structs or stuffing business logic into the entrypoint; lean on the event/port architecture to keep behavior modular.

---

## Usage Guidelines

**For AI Agents:**

- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge

**For Humans:**

- Keep this file lean and focused on agent needs
- Update when the technology stack or patterns change
- Review quarterly for outdated rules
- Remove rules that become obvious over time

Last Updated: 2026-01-04T15:15:06Z
