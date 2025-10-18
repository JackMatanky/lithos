# User Interface Design Goals

## Overall UX Vision

The CLI's user experience should be fast, intuitive, and discoverable, catering to keyboard-driven workflows. The design should minimize cognitive load by providing sensible defaults and clear feedback. The template engine must have robust error-handling capabilities to provide clear, actionable feedback to the user in case of a failure.

## Key Interaction Paradigms

- **Command-based:** The primary interaction will be through a set of clear and consistent commands (e.g., `new`, `index`).
- **Fuzzy Finding:** For interactive discovery of templates, a fuzzy finder will be used to provide a fast and forgiving search experience (inspired by fzf). This will be built as a core, reusable component that can be leveraged by multiple commands.
- **Interactive Prompts:** For templates requiring user input, the CLI will present prompts and suggesters directly in the terminal.

## Core Commands (MVP)

- `lithos new <template-name>`: Generates a new note from a specific template. If `<template-name>` is omitted, this command will automatically launch the fuzzy finder.
- `lithos find`: Explicitly opens an interactive fuzzy finder to search for and select a template to generate.
- `lithos index`: Manually triggers the re-indexing of the vault.

## Post-MVP Command Expansions

The following commands are planned for post-MVP releases and the initial architecture should accommodate their future addition:

- `lithos new <template> --output-dir <path>`: A flag to specify a different output directory for the generated note.
- `lithos list template`: Lists all available templates.
- `lithos list schema`: Lists all available schemas.
- `lithos schema validate <file>`: Validates an existing file against a schema.
- **Favorites/Pinning:** A mechanism to allow users to "favorite" or "pin" templates, which will cause them to appear at the top of the fuzzy finder list.
