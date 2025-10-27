# Epic 4: Interactive Input Engine

This epic delivers the interactive ports, adapters, TemplateEngine helpers, and CLI wiring defined in architecture v0.6.8. Completing it lets Lithos gather user input for templates and discover templates interactively through consistent PromptPort and FinderPort abstractions. Stories proceed from SPI contracts to adapter implementations and finally to TemplateEngine and CommandOrchestrator integration.

---

## Story 4.1 PromptPort Contract

As a developer,
I want the PromptPort interface and configuration structs defined per the architecture,
so that interactive prompts can be implemented behind a stable SPI.

**Prerequisites:** Epic 3 complete (QueryService available).

### Acceptance Criteria
1. `internal/ports/spi/prompt.go` declares `PromptPort` exactly as in `docs/architecture/components.md#promptport`, including GoDoc references to FR10 in `docs/prd/requirements.md`.
2. `PromptConfig` and `SuggesterConfig` structs expose the documented fields (Name, Label, Default, Options, Validator, NonInteractiveFallback) with inline comments referencing the architecture section.
3. Port documentation states error handling expectations (`InteractiveError`) per `docs/architecture/error-handling-strategy.md`.
4. `golangci-lint run ./internal/ports/spi` succeeds.

---

## Story 4.2 PromptUI Adapter

As a developer,
I want a PromptUI-based adapter that satisfies PromptPort,
so that templates can collect user input through the CLI.

**Prerequisites:** Story 4.1.

### Acceptance Criteria
1. `internal/adapters/spi/interactive/promptui.go` implements `PromptPort` using `github.com/manifoldco/promptui` exactly as in `docs/architecture/components.md#promptuiadapter`, including default handling and validator hooks.
2. Adapter detects non-TTY environments via `golang.org/x/term` and returns `InteractiveError` with remediation guidance per error-handling strategy.
3. Logging follows coding standards (debug for retries, info for prompt start/end) and satisfies FR10 UX guidance.
4. Unit tests cover successful prompt/suggester interactions, validation failures, cancellation, and non-interactive fallback.
5. `golangci-lint run ./internal/adapters/spi/interactive` and `go test ./internal/adapters/spi/interactive` succeed.

---

## Story 4.3 FinderPort & Fuzzy Adapter

As a developer,
I want FinderPort and its fuzzy finder adapter,
so that the CLI can provide interactive template selection as described in the architecture.

**Prerequisites:** Stories 4.1–4.2.

### Acceptance Criteria
1. `internal/ports/spi/finder.go` declares `FinderPort` exactly as in `docs/architecture/components.md#finderport` with GoDoc referencing FR10.
2. `internal/adapters/spi/interactive/fuzzyfind.go` implements the port using `github.com/ktr0731/go-fuzzyfinder`, including preview text support and non-TTY handling described in the architecture.
3. Adapter wraps cancellation and non-interactive scenarios in `InteractiveError` with consistent messaging.
4. Unit tests simulate selection success, cancellation, non-interactive mode, and ensure logging meets coding standards.
5. `golangci-lint run ./internal/adapters/spi/interactive` and `go test ./internal/adapters/spi/interactive` succeed.

---

## Story 4.4 TemplateEngine Interactive Helpers

As a developer,
I want TemplateEngine to expose the interactive helper functions,
so that templates can call `prompt`, `suggester`, and the documented path utilities.

**Prerequisites:** Stories 4.1–4.3.

### Acceptance Criteria
1. `internal/app/template/service.go` registers helper functions (prompt, suggester, now, path, folder, basename, extension, join, vaultPath, query helpers) exactly as documented in `docs/architecture/components.md#templateengine`.
2. Helpers delegate to PromptPort, FinderPort, QueryService, and Config consistently with FR3 requirements, without leaking infrastructure concerns.
3. Implementation exposes only the Go standard library packages approved in the architecture (strings, path, time) and documents any additions.
4. Unit tests with fake ports verify helper behaviour, path context handling, error propagation, and ensure interactive helpers respect non-interactive mode.
5. `golangci-lint run ./internal/app/template` and `go test ./internal/app/template` succeed.

---

## Story 4.5 CLI Find Command

As a developer,
I want CommandOrchestrator and the CLI adapter to offer the interactive `find` flow,
so that users can discover templates via FinderPort.

**Prerequisites:** Stories 4.1–4.4.

### Acceptance Criteria
1. `internal/app/command/orchestrator.go` implements `FindTemplates` delegating to TemplatePort and FinderPort exactly as in `docs/architecture/components.md#commandorchestrator`, returning structured errors for cancellation vs failure.
2. The CLI adapter registers a `find` command that invokes the orchestrator, prints results, respects non-interactive mode, and updates usage docs, satisfying FR2 and FR10.
3. Integration tests run `lithos find` with stubbed ports to confirm control flow, cancellation handling, and output formatting per coding standards.
