---
title: "Testing Policy"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Behavioral and quality policy for tests"
---

# Testing Policy

## Core rules

- One behavior per test; split tests when names imply multiple behaviors.
- Prefer explicit assertions with diagnostic context.
- Use verb-first test naming.
- Co-locate unit tests with implementation using `#[cfg(test)]`.
- Keep fixtures local and simple unless sharing is clearly beneficial.

## Quality posture

- Tests must be deterministic and isolated.
- Tests should validate success and failure paths for relevant behavior.
- Public APIs should include useful doc-tests where appropriate.

## Snapshot policy

- Snapshot testing is not the default approach.
- Prefer explicit assertions for critical logic and contracts.
