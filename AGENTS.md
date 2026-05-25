# Lithos - AI Agent Reference

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/` in this repository. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the canonical label vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Domain documentation uses a multi-context layout with a root `CONTEXT-MAP.md` that points to per-context `CONTEXT.md` files; ADRs are read per context as mapped. See `docs/agents/domain.md`.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **lithos** (19449 symbols, 25397 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/lithos/context` | Codebase overview, check index freshness |
| `gitnexus://repo/lithos/clusters` | All functional areas |
| `gitnexus://repo/lithos/processes` | All execution flows |
| `gitnexus://repo/lithos/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->

<!-- graphify:start -->
# graphify

- **graphify** (`~/.agents/skills/graphify/SKILL.md`) - any input to knowledge graph. Trigger: `/graphify`
When the user types `/graphify`, invoke the Skill tool with `skill: "graphify"` before doing anything else.
<!-- graphify:end -->

<!-- mise:start -->
# Mise — Environment & Task Orchestration

This project uses **mise** for tool versioning and task management. Use the Mise MCP tools to manage dependencies and execute project tasks.

> **Note**: Mise tools require `MISE_EXPERIMENTAL=1` to be enabled in the environment.

## Always Do

- **MUST check available tasks** using `mise://tasks` before assuming how to build, test, or lint the project.
- **MUST verify tool versions** using `mise://tools` if you encounter environment-specific issues.
- **ALWAYS prefer `run_task`** for executing project commands (build, test, fmt) instead of raw shell commands when a task exists.

## Never Do

- NEVER run a shell command that has an equivalent `mise` task (check `mise://tasks`).
- NEVER modify `.tool-versions` or `mise.toml` without verifying the impact on the environment.

## Resources

| Resource | Use for |
|----------|---------|
| `mise://tools` | List managed tools and their versions |
| `mise://tasks` | List all available project tasks (including those in `.mise/tasks/`) and dependencies |
| `mise://env` | View environment variables defined in mise |
| `mise://config` | View active mise configuration and project root |

## Tools

| Tool | Action |
|------|--------|
| `run_task` | Execute any mise task (e.g., `run_task({task: "test"})`). Runs both root tasks and those discovered in `.mise/tasks/`. |

### Common Tasks

| Task                   | Action                                                                            |
| :--------------------- | :-------------------------------------------------------------------------------- |
| `verify`               | Full quality gate orchestration (fmt + lint + tests + adr:validate) (alias: `v`). |
| `quality`              | Run all quality gates (fmt, lint, adr:validate) (alias: `q`).                     |
| `test`                 | Run all tests (unit, integration, e2e) (alias: `t`).                              |
| `lint`                 | Run clippy lints on all workspace crates (alias: `l`).                            |
| `fmt`                  | Format all Rust files in the workspace (alias: `f`).                              |
| `build`                | Build all workspace crates (alias: `b`).                                          |
| `clean`                | Clean workspace build artifacts and cache (alias: `c`).                           |
| `doc`                  | Generate and open project documentation.                                          |
| `adr:validate`         | Validate ADRs for template compliance.                                            |
| `test:unit`            | Run all unit tests using nextest (alias: `tu`).                                   |

<!-- mise:end -->

## Standards & Guidelines

- **Naming**: Follow [Naming Taxonomy](docs/naming-taxonomy.md)
- **Idioms**: Follow [Rust Unofficial Idioms](docs/refs/rust/rust_unofficial_idioms/)
- **Performance**: Adhere to [Zero-Copy & Crate Patterns](docs/refs/crates/) (rkyv, redb)
- **Dependencies**: Consult and update [Dependency Registry](docs/DEPENDENCIES.md) when adding, removing, or revisiting crates.
- **Type Safety**: Private fields by default, validation at construction, newtype wrappers for domain constraints.
- **Test-First**: Red-green-refactor cycle required - tests before implementation.

## Definition of Done

Before marking any task complete:
- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] All public APIs have tests (functions, methods, traits)
- [ ] Tests cover critical paths and business logic
- [ ] No `unwrap()`/`panic!` in production code
- [ ] Context boundaries respected (no cross-imports)
- [ ] Unified Repository pattern followed (single trait per context)
- [ ] Type-driven design applied (private fields, validated constructors)
- [ ] Documentation updated (doc comments for public APIs)
- [ ] ADR created if architectural decision made
- [ ] No string allocation anti-patterns (no `.to_owned().into()`, no unnecessary `.to_string()` in hot paths)
