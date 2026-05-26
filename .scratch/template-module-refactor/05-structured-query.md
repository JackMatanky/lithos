---
labels: ["needs-triage"]
---

## Parent

None

## What to build

Implement the engine-agnostic `QueryBuilder` API that mimics Dataview/SQL patterns for safe, structured data retrieval. Expose this builder to templates via `li.query(...)`. The template author should be able to construct queries dynamically during rendering to fetch related notes based on schema semantics.

## Acceptance criteria

- [ ] `QueryBuilder` and `QueryFilter` structured domain types are implemented.
- [ ] `TemplateRuntime` exposes `li.query(...)` allowing template authors to construct these objects mid-render.
- [ ] The queries correctly serialize/deserialize without leaking raw database strings into the template layer.
- [ ] Test verifies that a `minijinja` template can invoke `li.query("project").where(...)` and the runtime accurately maps it into the `QueryBuilder` struct.

## Blocked by

- .scratch/template/04-interactive-runtime.md
