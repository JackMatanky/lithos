# Raw Property Map emits hash index

- Label: `needs-triage`
- Type: `AFK`
- Category: `enhancement`
- State: `needs-triage`

## What to build

Refactor schema raw ingestion so `RawPropertyMap::compute_hashes()` produces `Blake3HashIndex<PropertyName>` directly, instead of a view-defined wrapper type. Keep this fully crate-internal and preserve existing incremental behavior.

## Acceptance criteria

- [ ] `RawPropertyMap::compute_hashes()` returns `Blake3HashIndex<PropertyName>`.
- [ ] No public API exposes `Blake3HashIndex` (it remains `pub(crate)`).
- [ ] Existing schema delta/property hashing behavior remains unchanged, verified by targeted tests.

## Blocked by

None - can start immediately.
