# Findings & Decisions: Bench Script Refactoring

## Requirements
<!-- User-specified priorities -->
- Better mise integration (leverage vars, env, task dependencies, outputs/sources)
- Better code organization (clearer separation of concerns, function structure)
- Preserve all existing functionality (4 modes, all flags/options)
- Maintain backward compatibility

## Research Findings

### Current Script Structure (347 lines)
- **Lines 1-19:** mise metadata (#MISE, #USAGE declarations)
- **Lines 20-47:** Package mapping helper (map shorthand → full package names)
- **Lines 48-60:** Baseline name generation
- **Lines 61-82:** Bench target discovery (find + awk)
- **Lines 83-137:** Argument builders (cargo args, bench args)
- **Lines 138-173:** Archive management (ensure dir, verify critcmp, export)
- **Lines 174-193:** Benchmark execution
- **Lines 194-211:** Baseline path resolution
- **Lines 212-257:** Command implementations (compare, list, open)
- **Lines 258-303:** Run command (main workflow)
- **Lines 304-328:** Validation logic
- **Lines 329-347:** Main entry point

### Mise Features Available
- **vars:** Define reusable variables in mise.toml (lines 55-78 show existing vars usage)
- **env:** Environment variables for tasks
- **depends:** Task dependencies (already used in mise.toml, e.g., line 88)
- **sources/outputs:** Track file dependencies and outputs for caching
- **tools:** Managed via mise (critcmp already added)
- **quiet/hide:** Control task visibility and output

### Current Pain Points
1. **Hardcoded paths:** `ARCHIVE_DIR=".benchmarks/baselines"` could be mise var
2. **Magic strings:** Package names ("core" → "lithos-core") scattered in logic
3. **Mixed concerns:** Validation, execution, archival all in one function (cmd_run)
4. **No task dependencies:** Could split into subtasks (run-bench, archive-baseline)
5. **Limited mise var usage:** Could leverage PROJECT_ROOT, vars for paths
6. **No outputs/sources tracking:** Mise can't optimize re-runs

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Approach A: Mise-First Decomposition | Best alignment with goals, leverages all mise features, clearest separation |
| NO orchestrator script | User feedback: violates Google Shell Style Guide, mise has better patterns |
| Single file, better organized | User feedback: if compare is only 50 lines, keep it simple |
| Leverage mise vars | Move config from hardcoded to mise.toml [vars] section (not [vars.bench]) |
| Use choices enum for mode | User feedback: use #USAGE arg with choices for compare/list/open modes |
| Add sources/outputs tracking | Enable mise caching for unchanged benchmarks |
| Organize by domain, not execution order | Group: config → utilities → validators → executors → mode handlers |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
|       |            |

## Resources
- mise.toml: `/Users/jack/Documents/41_personal/lithos/.worktrees/chore/bump-criterion-0.8.2/mise.toml`
- Current bench script: `.worktrees/chore/bump-criterion-0.8.2/.mise/tasks/test/bench`
- mise task docs: https://mise.jdx.dev/tasks/
- Existing task structure: `.mise/tasks/test/` (8 task files)

## Visual/Browser Findings
<!-- None yet -->
-

---
*Update this file after every 2 view/browser/search operations*
