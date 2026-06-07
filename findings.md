# Findings: Phase 2 Environment Discovery Architecture Refactor

## Review Corrections From User

- Discovery warnings must not be needed by `config/`. If they are, the implementation boundary is wrong.
- `GLOBAL_MARKER_FILES` should retain its name.
- `GLOBAL_MARKER_FILES` must include options for a `lithos` file at the base directory and a `lithos/config` file inside the base directory.
- `GlobalConfigProbe` naming is misleading inside `discovery/`; remove `Config` from the type name.
- `GlobalConfigProbe` should be renamed to `GlobalRootProbe` because it checks marker files in directory roots.
- Do not split `probe.rs`.
- Rename `FoundRootMarker` to `DiscoveredMarker`.
- `GlobalRootProbe` should have a plain `probe()` method like `VaultRootProbe`; `DiscoveryEngine` should handle global warnings similarly to how `resolve_ascending()` handles vault warnings.
- Config must not receive anything identifying source/method for vault/global paths. Config needs path/pathbuf and nothing else beyond mechanically necessary file metadata.
- Short-term fixes that preserve poor boundaries are unacceptable.
- `GlobalSourceType` should keep its current name.
- `GlobalDirectoryKind` should instead be named `GlobalSourceDirectory`.
- Do not create `SelectedConfigPath`; remove `location` from existing `DiscoveredConfigFile` so it becomes the path-only handoff shape.

## Current Code Findings

- `Builder::load()` currently passes `flag_path: Some(vault_root.as_path())` to `find_vault()`, but explicit source returns no marker.
- `ConfigDiscoveryResult::from_discovery()` currently consumes `VaultDiscoveryResult` and `GlobalDiscoveryResult`, which includes Discovery source identity and warnings.
- `config/root.rs` imports `GlobalSourceType`, coupling Config to Discovery source semantics.
- `config/location.rs` generates candidate paths through `LocalConfigLocation::candidate_path()`.
- `config/candidates.rs` probes filesystem for local config candidates.
- `GlobalConfigProbe` currently performs exact probing, case-correction scanning, warning creation, deduplication, and marker construction; the target is `GlobalRootProbe::probe()` for marker probing only, with warning orchestration in `DiscoveryEngine`.
- `GLOBAL_MARKER_FILES` currently only includes `lithos` and does not include `lithos/config`.

## Architectural Conclusion

The correct boundary is not “Discovery result maps directly to Config result.” The correct boundary is “Discovery selects paths; Builder passes selected path data to Config.” Discovery source and warnings should go to CLI/reporting, not Config.

`DiscoveredConfigFile` already represents the handoff concept once `location` is removed. Creating a new `SelectedConfigPath` would duplicate meaning and increase type churn without improving the boundary.

## GitNexus Findings From Review

- `find_vault` impact was HIGH: 19 direct callers, 3 affected flows, Discovery and Config modules impacted.
- `find_global` impact appeared LOW in indexed graph, but the index was stale relative to the branch implementation.
- `Builder::load` is the key integration caller for the vault discovery regression.

## Testing Findings

- Focused worktree command `mise run test:unit:core` passed with 1553 core unit tests after deleting obsolete config candidate/location modules.
- Full worktree command `mise run test` passed with unit, doc, integration, and e2e tests.
- Existing tests did not catch the `Builder::load()` local config bypass.
- Existing tests did not enforce Config/Discovery boundary purity.
- Existing tests did not cover `<base>/lithos/config.{toml,json,yaml,yml}` global marker form.

## Implementation Findings

- `find_vault()` explicit/env override branches needed to validate the override root and then run `VaultRootProbe`; this keeps source identity while fixing known-root marker discovery.
- `ConfigDiscoveryResult::from_discovery()` can ignore Discovery source and warnings entirely; selected marker path/base/format are sufficient for Config.
- `config/location.rs` and `config/candidates.rs` had no remaining production use after the path-only handoff and were removed rather than preserved as dead taxonomy.
- `GlobalRootProbe::probe()` can remain warning-free; `DiscoveryEngine` can compose exact markers with case-correction markers and warnings before selection.

## Follow-Up Audit Findings

- `DiscoveryPolicy` still exposes `precedence` rather than the planned `vault_precedence`, so vault/global precedence are not named symmetrically yet.
- `GlobalDiscoveryInput` still carries separate `xdg_config_base`, `user_config_base`, and `system_config_base` fields instead of a directory candidate slice.
- `GlobalSourceType` still mixes the environment file source and directory source variants; the plan calls for keeping the enum name while splitting directory identity through `GlobalSourceDirectory`.
- The known-vault-root behavior is fixed internally via explicit/env override probing, but there is not yet a dedicated API for a known vault root as described by Task 6.
- Architecture invariants around `config/` not importing `discovery::diagnostics`, Discovery source enums, or config-owned filesystem candidate generation are not yet directly locked by tests in this branch.

## Follow-Up Audit Resolution

- `DiscoveryPolicy::precedence` was renamed to `vault_precedence`, and `global_precedence` is now explicit.
- `GlobalDiscoveryInput` now accepts `env_file`, caller-provided `GlobalDirectoryCandidate` values, and `suppress`.
- `GlobalSourceType` keeps its name while wrapping `GlobalSourceDirectory` for directory-origin source identity.
- `DiscoveryEngine::find_known_vault()` now probes marker files for already-known vault roots, and `Builder::load()` uses that API.
- Architecture tests now lock Config away from Discovery diagnostics/source policy, prevent Config-owned candidate discovery modules, require directory-candidate global input, require symmetric policy names, and prevent `Builder` from hardcoding `/etc/lithos`.

## Architecture Invariants To Add

- `config/` must not import `discovery::diagnostics`.
- `config/` must not import Discovery source enums.
- `config/` must not perform filesystem candidate generation.
- Discovery probes should have one clear responsibility.
- Builder is allowed to orchestrate but should not convert source identity into Config semantics.

## Discovery Module Review Notes

- `lithos-core/src/discovery/CONTEXT.md` already defines the intended seam: Discovery locates runtime filesystem context before config loading, returns path/source/format metadata, and never imports Config.
- The current architectural issue is not only cross-imports; it is that Discovery-internal modules are not deep enough. `engine`, `probe`, `policy`, diagnostics, and marker handoff currently overlap on candidate generation, source meaning, warning production, and selected path representation.
- GitNexus query for Discovery concepts returned relevant definitions but poor process matching, suggesting the indexed execution-flow graph is not reliable for this branch-level Discovery review.
- `engine.rs` currently owns orchestration plus input models, result models, selected marker handoff, override validation, env-file global mechanics, directory-tier global mechanics, ascending walk assembly, candidate selection, and alternatives assembly. This makes `engine` broad rather than deep.
- `probe.rs` currently owns marker patterns, exact marker probing, global case-correction scanning, warning production, marker construction, deduplication, and the probe trait. This creates overlapping responsibilities between probing and diagnostics.
- `policy.rs` currently models vault source precedence only through `DiscoveryPolicy`, while global precedence is hardcoded through `GlobalSourceType::PRECEDENCE`.
- `walk.rs` is the cleanest current module: it owns bounded ascent and ceiling parsing. Its only overlap is that warning collection is tied directly to vault-specific diagnostics.
- `selector.rs` is narrow and deep enough: given markers, it selects by format/path precedence. It should remain pure and unaware of vault/global source semantics.
- ADR 009 is stale relative to current architecture but confirms that config source layering and runtime discovery were historically mixed. The refactor should prefer the current `Discovery -> Config -> Indexer` context model over reviving Figment-provider coupling.

## Independent Exploration Cross-Check

- A separate exploration pass confirmed `engine.rs` is overloaded with orchestration, DTO definitions, override validation, env-file probing, global directory probing, and result assembly.
- It confirmed `probe.rs` is overloaded with marker patterns, exact probing, global mis-case probing, deduplication, warning emission, and marker construction.
- It identified `FoundRootMarker` placement as a dependency smell: `probe.rs` and `selector.rs` import `engine::FoundRootMarker`, creating unnecessary coupling to orchestration.
- It noted `mod.rs` documents a `marker` module that does not exist.
- It identified warning typing asymmetry: vault results use bare `VaultDiscoveryWarning`, while global results use wrapped `DiscoveryWarning`.
- It suggested `GlobalConfigBaseProbe` as a precise rename, but final decision is `GlobalRootProbe`.

## Locked Design Corrections

- Keep `probe.rs` as one file. The responsibility cleanup happens inside the file by simplifying probe interfaces, not by splitting modules.
- `GlobalRootProbe` is the final replacement for `GlobalConfigProbe`.
- `FoundRootMarker` becomes `DiscoveredMarker`.
- `GlobalRootProbe` should mirror `VaultRootProbe`: a plain `probe()` method that returns markers.
- Global warning handling should move to `DiscoveryEngine`, analogous to `resolve_ascending()` owning warning collection while using `VaultRootProbe` for marker probing.

## Latest Review Decisions

- Case-correction applies to marker discovery generally, not only global discovery. Add a shared `case_correction_markers(patterns, base)` helper and use it from both vault and global flows.
- Do not add more methods to `DiscoveryProbe`; keep the probe trait exact-only and keep diagnostic marker discovery separate.
- Do not introduce a `DiscoverySelector` trait yet. The current need is shared pure selection/alternative assembly, not polymorphic dispatch.
- Replace duplicated marker selection/result assembly from `resolve_override()` and `resolve_ascending()` with shared selector helpers.
- `Builder` should not accept a known vault root for runtime loading; Discovery should find the vault root and Config should receive only selected config paths.
- `find_known_vault()` was a workaround for an incorrect Builder boundary and should not remain on the runtime path.
- `DiscoveryPolicy::global_precedence` should be `Vec<GlobalSourceType>` for symmetry with `vault_precedence: Vec<VaultSourceType>`.
- Global env path should be treated as a root directory source for marker discovery unless a distinct direct-file override source is explicitly added later.
- `validate_override()` should not grow into a vague mixed validator. Prefer explicit file/directory source validation if direct file sources are reintroduced.

## Latest Implementation Findings

- A `DiscoverySelector` trait is unnecessary for the current design; shared pure functions cover selection and alternative splitting without introducing dispatch.
- `find_known_vault()` hid the wrong boundary. Removing it forced Builder to call Discovery and made the constructor input a start directory rather than a pre-discovered vault root.
- `global_precedence: Vec<GlobalSourceType>` lets global source resolution mirror vault source resolution: iterate source identities, resolve each source to a candidate root, then probe markers.
- Direct global file override is now intentionally absent from this pass. If needed, it should be modeled as an explicit file source rather than overloading root-directory discovery.
- `case_correction_markers(patterns, base)` currently checks non-nested marker filenames in a base directory. Nested marker case correction remains a possible future extension if required.
- The vault case-correction behavior test is Linux-gated because case-insensitive macOS filesystems can resolve mis-cased exact paths before the correction path is observable.
