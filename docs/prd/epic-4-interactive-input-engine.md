# Epic 4: Interactive Input Engine

This epic brings the interactive user experience to life. It's about implementing the fuzzy finder for template discovery and adding the `prompt` and `suggester` functions to our template engine. This epic implements the InteractivePort interfaces and TemplateRepositoryPort following the hexagonal architecture patterns.

**Dependencies:** Epic 3 (Vault Indexing Engine)

## Story 4.1: Implement Interactive Testing Infrastructure

As a QA specialist, I want a testing infrastructure for interactive components following the testing strategy, so that I can write automated tests for prompts and suggesters.

### Acceptance Criteria

- 4.1.1: Testing approach follows `docs/architecture/testing-strategy.md#interactive-component-testing` with fake adapters for interactive components.
- 4.1.2: `internal/adapters/spi/interactive/` includes FakeInteractiveAdapter for testing purposes.
- 4.1.3: Unit tests can simulate user input and assert on terminal output without hanging.
- 4.1.4: Test harness integrates with table-driven test patterns specified in testing strategy.

## Story 4.2: Integrate Fuzzy Finder Library

As a developer, I want to integrate `go-fuzzyfinder` to create a proof-of-concept interactive selector, so that I can validate the library.

### Acceptance Criteria

- 4.2.1: `github.com/ktr0731/go-fuzzyfinder` is added to `go.mod`.
- 4.2.2: A temporary `lithos poc-fuzzy` command opens a full-screen fuzzy finder with a hardcoded list of strings.
- 4.2.3: The user can select an item, and the selected item is printed to the console.
- 4.2.4: The fuzzy finder UI should be a full-screen, interactive terminal interface, similar to the native `fzf`experience.

## Story 4.3: Add Placeholder `find` Command

As a developer, I want to add a `find` command to the CLI, so that its public API is defined before implementation.

### Acceptance Criteria

- 4.3.1: A `find` subcommand is added to Cobra.
- 4.3.2: Running `lithos find` prints a "Not Implemented" message.

## Story 4.4: Implement Template Repository Port and Adapter

As a developer, I want to implement the TemplateRepositoryPort and TemplateFSAdapter, so that template discovery follows the architectural patterns.

### Acceptance Criteria

- 4.4.1: `internal/ports/spi/` contains TemplateRepositoryPort interface with ListTemplates and GetTemplate methods per `docs/architecture/components.md#spi-port-interfaces`.
- 4.4.2: `internal/adapters/spi/template/` contains TemplateFSAdapter implementing TemplateRepositoryPort.
- 4.4.3: Adapter reads templates_dir from Config and returns Template domain models per `docs/architecture/data-models.md#template`.
- 4.4.4: Template discovery includes metadata extraction and lazy loading of template content.

## Story 4.5: Implement `find` Command Logic

As a user, I want to run `lithos find` to interactively select and generate a note from a template, so that I can easily find the template I need.

### Acceptance Criteria

- 4.5.1: `lithos find` calls `ListTemplates` to get templates.
- 4.5.2: The list of templates is passed to `go-fuzzyfinder`.
- 4.5.3: When a template is selected, the logic of `lithos new <selected-template>` is executed.
- 4.5.4: If no templates are found in the `templates_dir`, a "No templates found" message is displayed.

## Story 4.6: Implement Fuzzy Finding in `new` Command

As a user, I want to run `lithos new` without arguments to open a fuzzy finder, so that I have a convenient shortcut.

### Acceptance Criteria

- 4.6.1: `lithos new` checks if an argument was provided.
- 4.6.2: If no argument is provided, it executes the same logic as `lithos find`.

## Story 4.7: Integrate Prompt Library

As a developer, I want to integrate `promptui` to create a proof-of-concept interactive prompt, so that I can validate the library.

### Acceptance Criteria

- 4.7.1: `github.com/manifoldco/promptui` is added to `go.mod`.
- 4.7.2: A temporary `lithos poc-prompt` command displays a simple text prompt.
- 4.7.3: The user's input is captured and printed to the console.

## Story 4.8: Define Interactive Port Interfaces

As a developer, I want to define the InteractivePort interfaces, so that interactive functionality follows the hexagonal architecture patterns.

### Acceptance Criteria

- 4.8.1: `internal/ports/spi/` contains InteractivePort (PromptPort & FuzzyFinderPort) interfaces per `docs/architecture/components.md#spi-port-interfaces`.
- 4.8.2: Interfaces include Prompt, Suggester, and Find methods with proper configuration structures.

## Story 4.9: Implement Interactive CLI Adapter

As a developer, I want to implement the InteractiveCLIAdapter, so that interactive functionality is available through the CLI.

### Acceptance Criteria

- 4.9.1: `internal/adapters/spi/interactive/` contains InteractiveCLIAdapter implementing the interfaces.
- 4.9.2: Adapter uses the specified technology stack (`promptui`, `go-fuzzyfinder`) per `docs/architecture/tech-stack.md#interactive-libraries`.

## Story 4.10: Enhance Template Engine Service with Interactive Support

As a developer, I want to enhance the TemplateEngine to support interactive operations, so that template functions can access interactive capabilities through proper dependency injection.

### Acceptance Criteria

- 4.10.1: TemplateEngine accepts InteractivePort through constructor dependency injection.
- 4.10.2: Service creates template function map with closures wrapping InteractivePort calls.
- 4.10.3: CommandOrchestrator passes InteractiveCLIAdapter instance to TemplateEngine.
- 4.10.4: Template execution context includes interactive capabilities without tight coupling.

## Story 4.11: Implement Prompt Template Function

As a template author, I want to use `prompt()` function in templates, so that I can create interactive note generation experiences.

### Acceptance Criteria

- 4.11.1: TemplateEngine function map includes `prompt` function calling InteractivePort.Prompt method.
- 4.11.2: Helper functions (`list`) support template data structure creation following Go template patterns.

## Story 4.12: Implement Suggester Template Function

As a template author, I want to use `suggester()` function in templates, so that I can create interactive note generation experiences with selection options.

### Acceptance Criteria

- 4.12.1: `suggester` function calls InteractivePort.Suggester method with proper configuration handling.
- 4.12.2: Template functions handle both string slice and map inputs for suggester options.

## Story 4.13: Add Find Command Support

As a user, I want to use `lithos find` command to discover templates interactively, so that I can easily select templates through the proper CLI architecture.

### Acceptance Criteria

- 4.13.1: CobraCLIAdapter includes `find` subcommand calling CLICommandPort.Find method.
- 4.13.2: CommandOrchestrator implements Find method using TemplateRepositoryPort and InteractivePort.
- 4.13.3: Find command presents templates via fuzzy finder and executes selected template.
- 4.13.4: Template selection integrates with TemplateEngine for consistent execution flow.
