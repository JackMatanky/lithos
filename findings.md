# Findings - Clippy Semicolon Lints

## Lint Rationale
- `clippy::semicolon_outside_block`: Suggests moving the semicolon from a block’s final expression to the outside of the block. Rationale is consistency. (Restriction lint)
- `clippy::semicolon_inside_block`: Suggests moving the semicolon after a block to the inside of the block. Rationale is consistency. (Restriction lint)
- Both are mutually exclusive in their suggestions. Enabling both will cause them to conflict and oscillate.

## Conflicts and Confusing Behavior
- If both lints are enabled (e.g., via `clippy::restriction`), they will each suggest the opposite of the other.
- `semicolon_if_nothing_returned` (Pedantic) might also be involved if it expects a semicolon for expressions returning `()`.
- The `?` operator inside a block makes the block evaluate to the unwrapped value (often `()`). This makes the block an expression that technically "returns" a value, triggering these lints if followed by a semicolon or if the semicolon is inside.

## Idiomatic Solutions
- Avoid enabling both `semicolon_inside_block` and `semicolon_outside_block` simultaneously. They are restriction lints designed for projects to choose *one* style.
- For tests using `Result`, using blocks to scope variables is common. If the block returns `()`, the most common style in Rust (and default Clippy style if these restriction lints are off) is to have the semicolon inside the block if it's a series of statements, or simply not use a block if scoping isn't strictly necessary.
- If scoping *is* necessary, `{ op()?; }` (semicolon inside) is generally considered more idiomatic as it treats the block as a set of statements. However, `semicolon_outside_block` prefers `{ op()? };`.
- Recommendation: Choose one style and disable/ignore the other. If the project doesn't have a strong preference, disable both as they are "Restriction" lints which can be noisy.
