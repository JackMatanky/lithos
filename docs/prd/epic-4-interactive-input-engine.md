# Epic 4: Interactive Input Engine

**Epic Goal:** Deliver the interactive ports, adapters, and TemplateEngine integrations described in architecture v0.6.8 so Lithos can collect user input and perform template discovery from the CLI.

**Dependencies:** Epic 3 (Vault Indexing Engine)

**Architecture References:**
- `docs/architecture/components.md` v0.6.8 — TemplateEngine, PromptPort, FinderPort, CommandOrchestrator
- `docs/architecture/error-handling-strategy.md` v0.5.9 — InteractiveError
- `docs/architecture/coding-standards.md` v0.6.7 — logging, testing

---

## Story 4.1: Define PromptPort Interface & Config Types

As a developer, I want the PromptPort interface and configuration structs to match the architecture so TemplateEngine can delegate all interactive prompts.

### Acceptance Criteria
1. Create `internal/ports/spi/prompt.go` defining `PromptPort` with methods `Prompt(ctx context.Context, cfg PromptConfig) (string, error)` and `Suggester(ctx context.Context, cfg SuggesterConfig) (string, error)` exactly as in `components.md#promptport`.
2. Define `PromptConfig` and `SuggesterConfig` structs with the documented fields (Name, Label, Default, Options, Validator, etc.).
3. Document expected error handling (return `InteractiveError` when validation fails) referencing the error strategy doc.
4. Run `golangci-lint run ./internal/ports/spi`.

---

## Story 4.2: Implement PromptUI Adapter

As a developer, I need a PromptUI-based adapter that satisfies PromptPort while respecting TTY detection and validation rules.

### Acceptance Criteria
1. Implement `internal/adapters/spi/interactive/promptui.go` with constructor `NewPromptUIAdapter(log zerolog.Logger)`.
2. `Prompt` must render a promptui input, return user input, support default values, and enforce `PromptConfig.Validator` returning `InteractiveError` on failure.
3. `Suggester` must render a promptui select menu using the provided options, respecting search/filter behaviour described in the architecture notes.
4. Detect non-TTY environments and return `InteractiveError` indicating the operation is not interactive.
5. Unit tests cover successful prompt/suggester interactions (using promptui's runner in test mode), validation failure, and non-TTY fallback.
6. Run `golangci-lint run ./internal/adapters/spi/interactive` and `go test ./internal/adapters/spi/interactive`.

---

## Story 4.3: Define FinderPort & Fuzzyfind Adapter

As a developer, I need a FinderPort contract and adapter so the CLI can present the fuzzy template selection flow documented in the architecture.

### Acceptance Criteria
1. Declare `FinderPort` in `internal/ports/spi/finder.go` with method `Find(ctx context.Context, templates []Template) (Template, error)` per `components.md#finderport`.
2. Implement `internal/adapters/spi/interactive/fuzzyfind.go` using `github.com/ktr0731/go-fuzzyfinder` to satisfy the port, including preview text from template metadata.
3. Return `InteractiveError` when the environment is not interactive or when the user cancels the selection.
4. Unit tests simulate selection, cancellation, and non-interactive mode.
5. Run `golangci-lint run ./internal/adapters/spi/interactive` and `go test ./internal/adapters/spi/interactive`.

---

## Story 4.4: Extend TemplateEngine with Interactive Functions

As a developer, I want TemplateEngine to expose the interactive function map exactly as described in the architecture so templates can invoke `prompt`, `suggester`, and file path helpers.

### Acceptance Criteria
1. Update `internal/app/template/service.go` to register function map entries for `prompt`, `suggester`, `now`, `path`, `folder`, `basename`, `extension`, `join`, and `vaultPath` (see `components.md#templateengine`).
2. Ensure `prompt` and `suggester` delegate to PromptPort, `vaultPath` reads from Config, and file path helpers operate on the current execution path context.
3. Provide unit tests demonstrating each function (use fake PromptPort/FinderPort to return deterministic values).
4. Run `golangci-lint run ./internal/app/template` and `go test ./internal/app/template`.

---

## Story 4.5: Wire FinderPort through CommandOrchestrator & CLI

As a developer, I want the CLI `find` command to call CommandOrchestrator.FindTemplates via FinderPort so users can choose templates interactively.

### Acceptance Criteria
1. Implement `CommandOrchestrator.FindTemplates(ctx context.Context, query string) ([]Template, error)` delegating to FinderPort and TemplatePort as documented in `components.md#commandorchestrator`.
2. Update the CLI adapter to register a `find` command that calls the new method, prints results, and handles cancellation errors gracefully.
3. Add integration test(s) exercising `lithos find` with stubbed FinderPort to verify control flow.
4. Run `golangci-lint run ./internal/app/command ./internal/adapters/api/cli` and relevant `go test` packages.
