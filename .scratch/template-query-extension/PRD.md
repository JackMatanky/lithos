# Template Query Extension PRD

Status: draft

## Problem Statement

The Template foundation intentionally defers frontmatter, query/runtime objects, and `li.*` helpers. Users will eventually need Templates that read structured vault state, align generated output with Schema semantics, and produce frontmatter or note content from query results. Those features require careful boundaries across Template, Note, Schema, DB, and FS contexts.

Lithos needs a dedicated query extension phase so query behavior can be designed as a safe Template capability rather than a collection of ad hoc globals inside the rendering engine.

## Solution

Design a Template Query Extension that exposes structured, read-only query capabilities to Templates through controlled runtime objects and helpers. The extension should integrate with existing Note and Schema semantics without making the Template domain own note parsing, schema validation, or database mechanics.

The phase should also design frontmatter generation/handling rules for Templates, including how generated metadata aligns with Schema-defined Property semantics.

## User Stories

1. As a Lithos user, I want a Template to query notes, so that generated notes can include vault-derived information.
2. As a Lithos user, I want a Template to access structured note metadata, so that generated content can reflect existing frontmatter.
3. As a Lithos user, I want query helpers to respect Schema semantics, so that generated frontmatter remains valid.
4. As a Lithos user, I want query results to be deterministic for the same indexed vault state, so that renders are reproducible.
5. As a Lithos user, I want frontmatter generation to be explicit, so that note metadata is not mutated accidentally.
6. As a Lithos user, I want clear errors when a query is invalid, so that Template failures are actionable.
7. As a Lithos user, I want query helpers grouped under stable namespaces, so that Template source is readable.
8. As a Lithos user, I want query-powered Templates to remain read-only during render, so that queries cannot modify vault state.
9. As a Lithos user, I want generated frontmatter to merge only according to explicit future policy, so that foundation write safety is preserved.
10. As a developer, I want query capabilities outside Template foundation, so that rendering can be implemented before data access semantics.
11. As a developer, I want Template query helpers to consume existing repository/query ports, so that DB mechanics do not leak into Template source.
12. As a developer, I want query runtime objects to be Lithos-shaped, so that MiniJinja does not own the query domain model.
13. As a developer, I want frontmatter handling to align with Schema Property and Property Spec language, so that Template output respects existing domain terms.
14. As a developer, I want query errors to map to Template errors, so that service callers receive coherent failures.
15. As a developer, I want query tests independent from render-engine internals, so that query behavior can be verified against indexed fixtures.
16. As a maintainer, I want Template -> Schema interaction to remain planned and explicit, so that Template does not duplicate Schema validation logic.
17. As a maintainer, I want Template -> Note interaction to avoid cross-context imports that violate architecture tests.
18. As a maintainer, I want DB and FS access to remain behind existing infrastructure contexts, so that query helpers do not perform raw I/O.
19. As a maintainer, I want query helpers to be capability-gated, so that non-query render modes remain simple.
20. As a maintainer, I want merge-frontmatter policies deferred until explicitly designed, so that write behavior stays safe.

## Implementation Decisions

- Build this phase after Template foundation and extension registry.
- Treat query/frontmatter behavior as a Template extension capability, not foundation behavior.
- Keep Template domain models free of Note, Schema, DB, and FS implementation details.
- Use existing Note and Schema vocabulary: Note, frontmatter, Schema, Property, Property Spec, Property Bank Reference, and Resolved Schema.
- Design query helpers around read-only indexed state.
- Keep render-time queries deterministic for a fixed index snapshot.
- Avoid raw filesystem reads and writes in query helpers.
- Avoid DB implementation types in Template service or domain public APIs.
- Define frontmatter handling separately from single-output file commit behavior.
- Defer merge, append, overwrite, and conflict policies unless this PRD explicitly expands to cover them.
- Keep `li.*` or equivalent runtime namespaces stable and documented once chosen.

## Testing Decisions

- Query extension tests should use controlled indexed fixtures rather than live ad hoc filesystem state.
- Tests should cover successful note queries, empty query results, invalid query expressions, schema-aligned metadata output, and query error mapping.
- Determinism tests should verify stable output for the same indexed state.
- Architecture tests should verify context boundaries and absence of raw FS/DB leakage.
- Frontmatter tests should verify generated metadata shape without implying merge policies that are out of scope.
- Template rendering tests should verify query helpers work through the Template Engine without exposing MiniJinja internals.

## Out of Scope

- Template foundation ingestion, rendering, artifact commit pipeline, and minimal CLI.
- Extension registry mechanics unless needed to register query helpers.
- Prompt interaction and user selection flows.
- Arbitrary scripts, hooks, shell execution, or write-capable queries.
- Multi-file template packs.
- Merge-frontmatter, overwrite, append, rename, or conflict-resolution policies unless separately accepted.
- Reimplementing Note parsing or Schema validation inside Template.

## Further Notes

- This PRD is a draft and needs a dedicated grilling session because it crosses Template, Note, Schema, DB, and FS boundaries.
- The context map already marks Template -> Schema and Template -> FS relationships as planned.
- This phase should be careful not to break architecture tests that forbid cross-context imports.
