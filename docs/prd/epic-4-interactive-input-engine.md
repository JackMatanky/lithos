# Epic 4: Interactive Input Engine

This epic brings the interactive user experience to life. It's about implementing the fuzzy finder for template discovery and adding the `prompt` and `suggester` functions to our template engine.

## Story 4.1: Implement Test Harness for Interactive CLI Components

As a QA specialist, I want a reusable test harness for interactive terminal components, so that I can write automated tests for prompts and suggesters.

### Acceptance Criteria

- 4.1.1: A Go package for testing terminal interactions is chosen and implemented.
- 4.1.2: A simple test is created that successfully scripts a `promptui` interaction without hanging.
- 4.1.3: The test harness can simulate user input and assert on the terminal output.

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

## Story 4.4: Implement Template Lister Utility

As a developer, I want a utility to list all available template files, so that I have a data source for the fuzzy finder.

### Acceptance Criteria

- 4.4.1: An `engine/lister.go` file contains a `ListTemplates(config *Config)` function.
- 4.4.2: It reads `templates_dir` from the config.
- 4.4.3: It returns a slice of absolute paths to all templates.

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

## Story 4.8: Define Interactive Engine Interface

As a developer, I want to define an `Interactive` interface for terminal interactions, so that the template engine is decoupled from the specific library.

### Acceptance Criteria

- 4.8.1: An `interactive/interactive.go` file is created.
- 4.8.2: It contains an `Interactive` interface with `Prompt` and `Suggest` methods.
- 4.8.3: A `PromptUI_Backend` struct is created that implements this interface.

## Story 4.9: Pass Interactive Session to Template Engine

As a developer, I want to pass the `Interactive` session to the template engine, so that template functions can access it.

### Acceptance Criteria

- 4.9.1: The template rendering engine is refactored to accept an `Interactive` interface instance.
- 4.9.2: The `new` and `find`commands pass a `PromptUI_Backend` instance to the engine.

## Story 4.10: Implement `prompt()` Template Function

As a template author, I want to use a `prompt()` function to ask for free-text input during note generation.

### Acceptance Criteria

- 4.10.1: A `prompt` function is added to the template engine's function map.
- 4.10.2: It calls the `Prompt` method on the `Interactive` interface.
- 4.10.3: The user's input is correctly rendered in the template's output.

## Story 4.11: Implement `suggester()` Template Function

As a template author, I want to use a `suggester()` function to ask the user to select from a list of options.

### Acceptance Criteria

- 4.11.1: A `suggester` function is added to the template function map.
- 4.11.2: It calls the `Suggest` method on the `Interactive`interface.
- 4.11.3: A template with `{{ suggester "Select status" (list "A" "B") }}` works correctly.
- 4.11.4: A `list` helper function is added to the engine for creating simple string slices in templates.
