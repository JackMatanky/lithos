# Technical Assumptions

This section outlines the initial technical direction and library choices that will guide the architecture and implementation of Lithos.

## Core Philosophy

- **Standard Library First:** Leverage Go's powerful standard library wherever possible for tasks like file I/O, path manipulation, string handling, and time operations.
- **Minimal, High-Quality Dependencies:** Where external libraries are necessary, prefer well-established, high-performance packages that are actively maintained.
- **Interface-Driven Architecture:** Core components, particularly storage, must be implemented behind a Go interface to decouple logic from implementation, allowing for future flexibility.

## Go Packages & Libraries

| Category                | Library/Package                                              | Rationale                                                                                                                                    |
| ----------------------- | ------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| **CLI Framework**       | `github.com/spf13/cobra`                                     | Industry standard for building powerful and extensible CLIs. Chosen for its robust command structure and flag parsing.                       |
| **Configuration**       | `github.com/spf13/viper`                                     | Seamlessly integrates with Cobra, providing powerful and flexible configuration management from files, environment variables, and flags.     |
| **YAML**                | `github.com/goccy/go-yaml`                                   | `go-yaml` is chosen for its superior performance, critical for fast vault indexing.                                                          |
| **Markdown Processing** | `github.com/yuin/goldmark`                                   | Extensible markdown parser with AST access for frontmatter extraction, heading parsing, and template rendering. Enables future markdown features without custom development. |
| **Storage / Indexing**  | Custom file-based cache (MVP), `go.etcd.io/bbolt` (Post-MVP) | An initial custom cache minimizes dependencies. `bbolt` is favored for its read-optimized performance, which is ideal for the index.         |
| **Interactive Prompts** | `github.com/manifoldco/promptui`                             | Chosen for its clean API and good user experience for creating interactive prompts and selection lists.                                      |
| **Fuzzy Finder UI**     | `github.com/ktr0731/go-fuzzyfinder`                          | A drop-in, fzf-like interactive picker that provides the desired UX for the `find` command with minimal implementation overhead for the MVP. |
| **Logging**             | `github.com/rs/zerolog`                                      | A high-performance, structured logging library that is ideal for CLI applications.                                                           |
| **Release Tooling**     | `github.com/goreleaser/goreleaser`                           | The de-facto standard for automating Go project releases, simplifying cross-platform builds and distribution.                                |
