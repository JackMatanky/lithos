---
title: "Coverage"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Coverage tooling, thresholds, and usage"
---

# Coverage

## Primary command

```bash
mise run test:coverage
```

## Coverage policy

- Treat coverage as a quality signal, not an end goal.
- Prioritize business logic, validation rules, and error paths.
- Do not chase superficial 100% line coverage on boilerplate.

## Suggested thresholds

- Overall target: `>= 80%`
- Critical paths: aim for full behavioral coverage
- New/changed critical logic: require explicit tests in PR

## Tooling notes

- Current workflow uses tarpaulin via `mise` task orchestration.
- Keep output artifacts CI-visible where workflow is configured.
- Re-run coverage after major refactors and domain-model changes.
