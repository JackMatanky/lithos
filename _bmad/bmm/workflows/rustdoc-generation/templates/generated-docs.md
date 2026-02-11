---
name: "generated-docs"
description: "Template for generated rustdoc documentation"
---

# Generated Documentation Template

```markdown
---
project: {project_name}
target: [target_path]
date: [current date]
mode: create
components: [counts]
compliance: rfc1574
---

# Generated Rustdoc Documentation

## Crate Documentation

[//! docs]

## Module Documentation

[//! docs for each module]

## Type Documentation

[/// docs for structs and enums]

## Function Documentation

[/// docs for functions and methods]

## Trait Documentation

[/// docs for traits]

## Validation Report

[compliance status and issues]

## Application Instructions

1. Copy doc comments into your source files
2. Run `cargo doc --open` to preview HTML
3. Run `cargo test --doc` to verify examples
```
