---
name: selection-of-template-engine-for-markdown-based-templates
status: accepted
stakeholders: [Jack (Developer), Architects]
date_proposed: 2026-01-08
date_decided: 2026-01-11
date_implemented: 2026-01-11
date_updated: 2026-03-11
update_reason: Clarified why runtime loading is required (template pack ecosystem), added Askama comparison, documented zero-copy integration strategy with rkyv
---

# ADR 007: Selection of Template Engine for Markdown-based Templates

## Context

Lithos templates are primarily Markdown files containing YAML frontmatter and body content with embedded logic. The engine must support high-frequency rendering for a Command Line Interface (CLI) and a future Language Server Protocol (LSP).

Key technical challenges for Markdown templates include:

1.  **Frontmatter Integrity**: Template logic must render within YAML blocks without breaking indentation or syntax (strict whitespace control).
2.  **Markdown Body Safety**: Special characters (`#`, `*`, `[`, `]`, `<`, `&`) must be handled without accidental HTML escaping that would break Markdown rendering.
3.  **Complex Logic in Docs**: Rendering loops and conditionals inside whitespace-sensitive structures like tables and lists.
4.  **Performance**: The LSP requires sub-50ms rendering for thousands of small snippets during real-time interaction.

## Decision

We will use **MiniJinja** as the primary template engine for Lithos Rust.

### 1. Engine Specifics for Markdown

- **Configurable Auto-Escaping**: Unlike Tera, which defaults to HTML escaping, MiniJinja allows us to configure the `Environment` with a custom auto-escape callback. We will disable escaping for `.md` and `.yaml` templates by default while maintaining the ability to mark specific data as `Safe`.
- **Whitespace Control**: Full support for Jinja2-style whitespace stripping (`{%- ... -%}`), essential for preserving YAML frontmatter alignment and Markdown table pipes.
- **Low-Overhead Snippets**: Optimized for fast compilation and rendering, making it ideal for the real-time feedback loops required by the LSP.

## Alternatives Considered

| Feature                | **MiniJinja**       | **Tera**      | **Handlebars**   | **Askama**        |
| :--------------------- | :------------------ | :------------ | :--------------- | :---------------- |
| **Performance (LSP)**  | **Ultra-High**      | High          | Medium-High      | Highest (compile) |
| **Binary Size**        | **Minimal (~50KB)** | Moderate      | Moderate         | None (compiled)   |
| **Markdown Escaping**  | **Native/Custom**   | Strict (HTML) | Helper-Dependent | Configurable      |
| **Whitespace Control** | **Excellent**       | Good          | Limited          | Excellent         |
| **Logic Capability**   | High (Jinja2)       | High (Jinja2) | Low (Logic-less) | High (Jinja2-like)|
| **Runtime Loading**    | **Yes**             | Yes           | Yes              | No (compile-time) |
| **User-Created**       | **Yes**             | Yes           | Yes              | No                |
| **Git Distribution**   | **Yes**             | Yes           | Yes              | No                |

### Why not Tera?

Tera's strict HTML-centric defaults are cumbersome for a Markdown tool. Disabling auto-escaping in Tera is globally handled and less flexible than MiniJinja's environment-level callbacks. MiniJinja's smaller binary footprint and faster rendering better align with our "Mechanical Sympathy" goals.

### Why not Handlebars?

Handlebars' "logic-less" philosophy requires excessive custom helpers for common PKM operations, such as conditionally formatting list items or complex schema-driven table generation. This leads to a fragmented and verbose template codebase.

### Why not Askama?

While Askama offers compile-time type safety and zero runtime overhead through compile-time template compilation, it **fundamentally conflicts with Lithos's core value proposition**:

**Template Pack Ecosystem Requirements (FR34-FR37):**
- Users must create custom templates in their editor (FR1)
- Templates must be shareable via Git repositories (FR34)
- Community members discover and adopt template packs (FR35)
- Template packs are distributed as `.jinja2` files, not embedded in binary

**Askama's Limitations:**
- Templates must be embedded in the binary at compile time
- Users cannot create or modify templates without recompiling Lithos
- Git distribution of template files is meaningless (templates aren't loaded at runtime)
- Community template pack ecosystem becomes impossible

**Why Runtime Loading is Non-Negotiable:**
Lithos is a **platform for user-created content**, not a fixed-template system. The entire product vision (Journey 3: Jordan distributes template packs, Journey 4: Maya adopts community templates) depends on runtime template loading. Choosing a compile-time engine would eliminate the core differentiator that makes Lithos valuable.

## Technical Validation

### Research Findings

- **Markdown Safety Advantage**: MiniJinja allows defining "Safe" vs "Raw" strings at the application level. Lithos can pass schema-validated Markdown snippets into a template and ensure they aren't mangled by the engine, while still protecting against accidental injection in other contexts.
- **Mechanical Sympathy**: MiniJinja's VM-based approach and minimal dependency tree (minimal binary size increase of ~50KB) align with Rust's performance goals.
- **Whitespace Control**: Jinja2-style `{%- ... -%}` is superior for Markdown where extra newlines can change document meaning.

### Compatibility & Performance

- **Hexagonal Alignment**: Wrapped in an SPI port, allowing the domain to define template logic without being bound to Jinja2 syntax if we ever need to switch.
- **Performance Impact**: The LSP requires sub-50ms rendering for thousands of small snippets; MiniJinja's low-overhead compilation meets this requirement.

### Integration with Zero-Copy Architecture (rkyv)

While MiniJinja is a runtime engine (templates compile on-demand), we maintain zero-copy performance through **typed template contexts**:

**Pattern: Zero-Copy Context Building**

```rust
// Template context uses borrowed references from archived data
#[derive(serde::Serialize)]
pub struct NoteTemplateContext<'a> {
    // Zero-copy borrows from archived note
    title: &'a str,
    date: SystemTime,
    tags: Vec<&'a str>,

    // Linked notes built from archived data
    linked_notes: Vec<LinkContext<'a>>,
}

// Build context from archived note (zero-copy reads)
storage.with_archived(note_id, |archived_note| {
    let ctx = NoteTemplateContext {
        title: archived_note.title(),           // Borrowed from archived
        date: archived_note.created_at(),       // Copy (SystemTime is cheap)
        tags: archived_note.tags()
            .iter()
            .map(|t| t.as_str())                // Borrowed from archived
            .collect(),
        linked_notes: build_link_contexts(archived_note),  // Zero-copy construction
    };

    // Only allocation: final template output string
    template.render(&ctx)
})?;
```

**Performance Characteristics:**
- **Zero-copy reads** from database via `rkyv::Archived<Note>`
- **Minimal allocations** (only for template output and context Vec/String wrappers)
- **Closure-based scope** ensures archived data lifetime safety
- **Sub-50ms rendering** achieved through efficient context building + MiniJinja VM

**Why This Works:**
1. Template contexts are **ephemeral** (created per-render, dropped immediately)
2. Context fields **borrow** from archived data (no deep clones)
3. MiniJinja serialization uses `serde::Serialize` (works with references)
4. Final output is the **only heap allocation** (unavoidable for user-facing string)

This approach gives us:
- ✅ Runtime template loading (user-created, Git-distributed)
- ✅ Zero-copy database reads (rkyv archived types)
- ✅ Minimal allocations (only for final output)
- ✅ LSP-compatible performance (<50ms)
- ✅ Type-safe template contexts (compile-time field checking)

## Consequences

### Positive

- **Runtime Flexibility**: Users can create, modify, and share templates without recompiling Lithos
- **Template Pack Ecosystem**: Enables Git-based distribution of community-created template packs (core product differentiator)
- **High Performance**: Sub-50ms rendering via efficient VM + zero-copy context building from rkyv archived data
- **Small Binary**: Minimal dependency footprint (~50KB increase)
- **Intuitive Syntax**: Jinja2 familiarity reduces learning curve for power users
- **Whitespace Control**: Essential for YAML frontmatter and Markdown table preservation
- **Zero-Copy Integration**: Template contexts borrow from archived database reads (minimal allocations)

### Negative

- **Runtime Template Errors**: Syntax errors discovered at execution time (not compile time)
  - *Mitigation*: Template validation commands (`lithos template check`)
  - *Mitigation*: LSP provides real-time syntax checking during template editing
- **Custom Environment Management**: Requires extension-based escaping configuration
  - *Mitigation*: Centralized `TemplateEnvironment` factory in `template/` context
- **Context Serialization Overhead**: Must serialize context to serde-compatible types
  - *Mitigation*: Use borrowed references (`&str`) to minimize allocations
  - *Mitigation*: Profile and optimize context building (likely < 10ms overhead)

### Trade-offs Accepted

We **explicitly choose** runtime loading over compile-time safety because:
1. Template pack ecosystem is a **core product requirement** (FR34-FR37)
2. User-created templates are **non-negotiable** for the target audience
3. Runtime errors are **acceptable** with proper tooling (validation commands, LSP)
4. Performance cost is **minimal** (<50ms total, <10ms for context building)

### Performance Optimization Strategy

1. **Zero-Copy Reads**: Use `with_archived()` to access database without deserialization
2. **Borrowed Contexts**: Template contexts use `&str` references to archived data
3. **Template Caching**: MiniJinja caches compiled templates (parse once, render many)
4. **Lazy Loading**: Load templates on-demand, not at startup
5. **Profiling**: Criterion benchmarks verify <50ms end-to-end rendering
