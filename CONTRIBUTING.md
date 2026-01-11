# Contributing to Lithos

We welcome contributions to Lithos! As a project focused on quality and architectural integrity, we have a few guidelines to ensure the codebase remains maintainable.

## Getting Started

1.  **Fork and Clone**: Fork the repository and clone it locally.
2.  **Setup**: Run `mise run dev-setup` to bootstrap your environment.
3.  **Explore**: Check out `_bmad-output/planning-artifacts/architecture.md` to understand the system design.

## Development Workflow

-   **Hexagonal Boundaries**: Respect the crate boundaries. `domain` must remain pure.
-   **Quality Gates**: All code must pass `mise run verify`.
-   **Tests**: Write unit tests for logic and integration tests for adapters.
-   **Commits**: Use conventional commits.

## Submitting Changes

1.  Create a feature branch.
2.  Implement your changes with tests.
3.  Ensure `mise run verify` passes.
4.  Submit a Pull Request with a clear description of the "why".

## Architectural Decisions

Significant changes (new dependencies, structural shifts) require an ADR (Architectural Decision Record). Please discuss these in an issue before implementation.

---

*Thank you for helping make Lithos better!*
