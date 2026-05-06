# Research: Lifetimes in Raw DTOs for High-Performance Ingestion

This document outlines best practices for implementing lifetimes in 'Raw' Data Transfer Objects (DTOs) within a parsing/ingestion pipeline, specifically targeting the `RawNote<'a>` implementation in Lithos.

## 1. `Cow<'a, str>` vs `&'a str` vs `String`

When building a zero-copy or low-allocation pipeline, selecting the right string representation is critical.

### `&'a str` (Strictly Borrowed)
- **Use Case:** Best for raw slices of the source buffer that never require modification (e.g., raw frontmatter blocks, raw headings before trimming).
- **Pros:** Highest performance; zero allocation.
- **Cons:** Extremely rigid. Cannot handle data that needs unescaping (e.g., `&lt;` -> `<`) or sanitization without a new allocation.

### `Cow<'a, str>` (Clone-on-Write)
- **Use Case:** The "gold standard" for DTOs.
- **Pros:** Remains `Borrowed(&'a str)` for the majority of cases where the source is already in the correct format. Seamlessly switches to `Owned(String)` when processing (like unescaping) is required.
- **Cons:** Slightly larger memory footprint (24-32 bytes vs 16 bytes for `&str`) due to the enum discriminant and owned buffer capacity.

### `String` / `Box<str>` (Owned)
- **Use Case:** Best for data that is entirely new or fundamentally transformed (e.g., UUIDs, hashes, or identifiers generated during parsing).
- **Pros:** Simplest ergonomics; no lifetimes.
- **Cons:** Forces an allocation for every string, which can be thousands of allocations in a large-scale ingestion task.

**Recommendation:** Use `&'a str` for structural metadata and `Cow<'a, str>` for content fields.

---

## 2. Managing 'Lifetime Infection'

Lifetimes "infect" everything they touch. If `RawNote` has a lifetime, any struct holding it (like `NoteIngestor` or a `Loader`) must also carry that lifetime.

### Mitigation Strategies

1. **Short-Lived Raw Phase:**
   The "Raw" DTO should only exist during the ingestion phase. The `Loader` should parse the file, produce a `RawNote<'a>`, and immediately map it to a `Note` (the owned domain entity).

2. **Self-Referential Bundling:**
   If the `RawNote` must be passed through many layers, bundle it with its source buffer using a crate like `self_cell`. This effectively "hides" the lifetime by creating an owned container.

```rust
use self_cell::self_cell;

self_cell!(
    pub struct RawNoteBundle {
        owner: String, // The source buffer
        #[covariant]
        dependent: RawNote, // RawNote<'a> borrowing from String
    }
);
```

---

## 3. Use of 'Boxed' vs 'Borrowed' in `Cow`

While `Cow<'a, str>` is standard, sometimes `Cow<'a, Box<str>>` is seen.

- **`Cow<'a, str>`:** The enum is `Borrowed(&'a str)` or `Owned(String)`. This is the most idiomatic and flexible.
- **`Cow<'a, Box<str>>`:** Adds an extra level of indirection for the owned case. Avoid this unless you are strictly optimizing for storage density in the owned case and have thousands of them.

**Recommendation:** Stick to `Cow<'a, str>`.

---

## 4. Idiomatic 'Cloning to Owned'

When a borrowed DTO needs to be stored or sent across a boundary that doesn't support its lifetime, it must be "owned."

### Custom `into_owned` Method
The most idiomatic way is to provide a method that returns the `static` version of the struct.

```rust
impl<'a> RawNote<'a> {
    pub fn into_owned(self) -> RawNote<'static> {
        RawNote {
            path: self.path,
            frontmatter: self.frontmatter.map(|f| f.into_owned()),
            headings: self.headings.into_iter().map(|h| h.into_owned()).collect(),
            // ...
        }
    }
}
```

---

## 5. Performance Implications of Nested Structs

Nested structs with lifetimes (e.g., `RawNote<'a> -> RawTask<'a> -> RawTag<'a>`) have **zero CPU overhead** at runtime. The compiler uses lifetimes for static analysis only.

However, they do have a "complexity cost":
- Every nested type must declare the lifetime.
- Generics and traits involving these types become more verbose.

**Recommendation:** Don't fear nesting lifetimes; the performance gains from zero-copy parsing in a "hot" ingestion pipeline are substantial.

---

## 6. Self-Referential Structures

If you need to keep the `RawNote` and the source buffer together:

- **`self_cell`:** Best for most cases. It's safe, efficient, and generates clean code.
- **`ouroboros`:** Extremely powerful but has a more complex API and higher compile-time cost.
- **Manual `unsafe`:** Not recommended unless `self_cell` cannot handle your structure.

---

## 7. High-Performance `RawNote<'a>` Example

```rust
use std::borrow::Cow;
use crate::note::position::{SourceByteOffset, SourceByteRange};

/// High-performance raw note using lifetimes for zero-copy parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct RawNote<'a> {
    pub path: NotePath,
    pub source_hash: Box<str>,
    /// Frontmatter text is a direct borrow from the source.
    pub frontmatter: Option<RawFrontmatter<'a>>,
    pub tags: Vec<RawTag<'a>>,
    pub tasks: Vec<RawTask<'a>>,
    // ...
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawFrontmatter<'a> {
    /// Direct slice from source.
    pub text: &'a str,
    pub range: SourceByteRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawTag<'a> {
    /// Cow allows for unescaped tags or direct borrows.
    pub value: Cow<'a, str>,
    pub position: SourceByteOffset,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawTask<'a> {
    /// Task text may require unescaping (Cow).
    pub text: Cow<'a, str>,
    pub tags: Vec<RawTag<'a>>,
    pub range: SourceByteRange,
}

impl<'a> RawNote<'a> {
    /// Convert the borrowed note into an owned version.
    pub fn into_owned(self) -> RawNote<'static> {
        RawNote {
            path: self.path,
            source_hash: self.source_hash,
            frontmatter: self.frontmatter.map(|f| RawFrontmatter {
                text: f.text.to_string().leak(), // Simple static conversion for example
                range: f.range,
            }),
            // Real implementation would handle this more robustly
            tags: self.tags.into_iter().map(|t| RawTag {
                value: Cow::Owned(t.value.into_owned()),
                position: t.position,
            }).collect(),
            tasks: self.tasks.into_iter().map(|t| RawTask {
                text: Cow::Owned(t.text.into_owned()),
                tags: t.tags.into_iter().map(|tag| RawTag {
                    value: Cow::Owned(tag.value.into_owned()),
                    position: tag.position,
                }).collect(),
                range: t.range,
            }).collect(),
        }
    }
}
```

## Summary Guidelines

1. **Prefer `Cow<'a, str>`** for any content field that might need unescaping.
2. **Use `&'a str`** for fields that are guaranteed to be direct slices.
3. **Limit the scope** of types with lifetimes to the ingestion phase.
4. **Use `self_cell`** if you must bundle the source and the DTO.
5. **Always provide an `into_owned` path** for long-term storage or multi-threaded processing.
