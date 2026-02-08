# Rustdoc Standards Reference

## Quick Reference by Component Type

### Crate Level (`lib.rs` / `main.rs`)

**Syntax:** Inner doc comments `//!`

**Structure:**
```rust
//! [One-line summary]
//!
//! [Detailed description]
//!
//! # Features
//!
//! - [Feature 1]
//! - [Feature 2]
//!
//! # Usage
//!
//! ```
//! use crate::Type;
//!
//! let instance = Type::new();
//! ```
```

**Checklist:**
- [ ] `//!` syntax (not `///`)
- [ ] Summary line (search result text)
- [ ] Features section (if applicable)
- [ ] Usage example with `use`
- [ ] Layout section (optional)

---

### Module Level

**Syntax:** Inner doc comments `//!` at file top

**Structure:**
```rust
//! [One-line summary]
//!
//! [High-level context]
//!
//! This module provides:
//! - [`Type`] - [Brief]
```

**Checklist:**
- [ ] `//!` at file top
- [ ] NOT inside `mod { }` blocks
- [ ] Summary line
- [ ] High-level only (types document themselves)
- [ ] Cross-references to related modules

---

### Structs

**Syntax:** Outer doc comments `///`

**Structure:**
```rust
/// [What it represents]
///
/// [When to use]
///
/// # Examples
///
/// ```
/// use crate::Struct;
///
/// let s = Struct::new();
/// ```
pub struct Struct {
    /// [Field description]
    pub field: Type,
}
```

**Checklist:**
- [ ] `///` syntax
- [ ] Summary: what it represents
- [ ] All public fields documented
- [ ] # Examples section
- [ ] # Panics (if applicable)
- [ ] # Errors (if applicable)

---

### Enums

**Syntax:** Outer doc comments `///`

**Structure:**
```rust
/// [What it represents]
///
/// # Examples
///
/// ```
/// use crate::Enum;
///
/// match value {
///     Enum::First => {},
///     Enum::Second => {},
/// }
/// ```
pub enum Enum {
    /// [When this variant is used]
    First,
    /// [Description]
    ///
    /// The data represents [what].
    Second(Data),
}
```

**Checklist:**
- [ ] `///` syntax
- [ ] Summary: what it represents
- [ ] All variants documented
- [ ] Data variants explain data
- [ ] # Examples with match patterns
- [ ] Edge case examples (None, Err)

---

### Functions & Methods

**Syntax:** Outer doc comments `///`

**Structure:**
```rust
/// [Third-person singular summary]
///
/// [Detailed explanation if needed]
///
/// # Panics
///
/// [If applicable]
///
/// # Errors
///
/// [For Result types]
///
/// # Safety
///
/// [For unsafe fn]
///
/// # Examples
///
/// ```
/// use crate::function;
///
/// let result = function()?;
/// ```
pub fn function() -> Result<T> {}
```

**Checklist:**
- [ ] `///` syntax
- [ ] Summary: third-person singular ("Returns")
- [ ] NO "Parameters:" section
- [ ] NO "Returns:" section
- [ ] # Examples section (REQUIRED)
- [ ] # Panics (if applicable)
- [ ] # Errors (for Result)
- [ ] # Safety (for unsafe)
- [ ] Examples use `?` not `unwrap()`

---

### Traits

**Syntax:** Outer doc comments `///`

**Structure:**
```rust
/// [What behavior is enabled]
///
/// [Contract for implementors]
///
/// # Examples
///
/// ```
/// struct MyType;
///
/// impl Trait for MyType {
///     fn method(&self) {}
/// }
/// ```
pub trait Trait {
    /// [Method description]
    fn method(&self);
}
```

**Checklist:**
- [ ] `///` syntax
- [ ] Summary: enabled behavior
- [ ] Contract documented
- [ ] All methods documented
- [ ] # Examples with implementation
- [ ] Required vs provided methods noted

---

## RFC 1574 Common Headings

Always use these exact heading texts:

| Heading | Use When |
|---------|----------|
| `# Examples` | Always (plural even if one) |
| `# Panics` | Function can panic |
| `# Errors` | Function returns Result |
| `# Safety` | unsafe fn or unsafe trait |
| `# Aborts` | Rare - process abort |
| `# Undefined Behavior` | unsafe with UB risk |

**Anti-Patterns:**
- ❌ `# Example` (use plural `# Examples`)
- ❌ `## Examples` (use single `#`)
- ❌ `Parameters:` (never use)
- ❌ `Returns:` (never use)

---

## Intra-Doc Links

**Syntax:**
```rust
/// The [`String`] passed in...
/// See [`crate::module::Type`]
///
/// [`String`]: ../string/struct.String.html
```

**Patterns:**
- Same type: `[`method`]: #method.method_name`
- Same module: `[`Type`]: struct.Type.html`
- Parent module: `[`Type`]: ../enum.Type.html`
- Child module: `[`Type`]: child/struct.Type.html`

---

## Doc Test Best Practices

**Basic Example:**
```rust
/// ```
/// let x = 5;
/// assert_eq!(x, 5);
/// ```
```

**With Error Handling:**
```rust
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let parsed = "42".parse::<i32>()?;
/// # Ok(())
/// # }
/// ```
```

**Ignored (non-compiling):**
```rust
/// ```ignore
/// this_code_wont_compile();
/// ```
```

---

## Validation Checklist

Before finalizing documentation:

- [ ] All public items documented
- [ ] Crate uses `//!`
- [ ] Modules use `//!` at top
- [ ] Types use `///`
- [ ] Functions use `///`
- [ ] Summary lines are clear
- [ ] Examples sections present
- [ ] Third-person singular for functions
- [ ] No "Parameters:" sections
- [ ] No "Returns:" sections
- [ ] Panics documented where applicable
- [ ] Errors documented for Result types
- [ ] Safety documented for unsafe
- [ ] Intra-doc links work
- [ ] `cargo doc` generates without errors
- [ ] `cargo test --doc` passes
