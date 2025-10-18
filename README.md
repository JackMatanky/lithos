# Lithos

Lithos is a Go-based command-line tool that brings Obsidian-style templating, schema validation, and metadata automation to any terminal workflow.

## Prerequisites

- Go 1.25 or later (minimum 1.23 as per architecture standards)
- [`just`](https://github.com/casey/just) task runner
- Git tooling for cloning and contribution
- Optional: `golangci-lint` (fetched automatically by provided tasks)

Verify prerequisites with:

```bash
go version
just --version
```

## Getting Started

1. **Clone the repository**

   ```bash
   git clone <repo-url> && cd lithos
   ```

2. **Bootstrap dependencies**

   ```bash
   just bootstrap
   ```

   The bootstrap recipe pins tool versions and validates your Go toolchain.

3. **Validate the workspace**

   ```bash
   just verify
   ```

   Runs formatting, linting, and `go test ./...` to confirm the environment is healthy.

## Quickstart Commands

| Command        | Purpose                                                    |
|----------------|------------------------------------------------------------|
| `just --list`  | Discover available automation tasks.                       |
| `just build`   | Compile the CLI into `./bin/lithos`.                       |
| `just test`    | Execute the unit test suite.                               |
| `just lint`    | Run `golangci-lint` with project rules.                    |
| `just verify`  | Format, lint, and test in a single pass (recommended).     |

During development, follow the epic/story sequence outlined in `docs/prd.md`. Coding standards, architecture, and testing expectations live in `docs/architecture.md`.

## Repository Structure

```
cmd/           CLI entry point (`main.go`)
internal/      Hexagonal core (domain, app services, ports, adapters)
justfile       Automation for builds, linting, and tests
docs/          PRD, architecture, elicitation artifacts
testdata/      Vault fixtures and golden files for automated tests
```

Refer to the PRD epics to understand feature order and dependencies before starting implementation work.
