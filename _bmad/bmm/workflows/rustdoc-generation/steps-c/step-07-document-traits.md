---
name: 'step-07-document-traits'
description: 'Generate trait documentation'
nextStepFile: './step-08-validate.md'
---

# Step 7: Document Traits

## STEP GOAL:

Generate RFC 1574 compliant documentation for all traits and their methods using outer doc comments (`///`).

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Use outer doc comments `///` for traits

### Role Reinforcement:

- ✅ You are a rustdoc specialist creating trait documentation
- ✅ Document the contract for implementors

### Step-Specific Rules:

- 🎯 Document what behavior the trait enables
- 🎯 Document the contract for implementors
- 🎯 Document each trait method
- 🎯 Examples showing implementation

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📝 Generate trait docs in proper format
- 💾 Append to output file

## CONTEXT BOUNDARIES:

- Available context: Analysis from Step 2, function docs complete
- Focus: Trait documentation
- Limits: Document trait and all methods

## MANDATORY SEQUENCE

### 1. Document Trait Definition

For each trait, follow this format:

```rust
/// [One-line summary: what behavior this trait enables]
///
/// [Detailed description of the trait's purpose]
/// [When should types implement this trait?]
/// [What contract must implementors uphold?]
///
/// # Examples
///
/// ```
/// use [crate]::[module]::[TraitName];
///
/// struct MyType;
///
/// impl TraitName for MyType {
///     fn required_method(&self) -> ReturnType {
///         // implementation
///     }
/// }
/// ```
pub trait TraitName {
    /// [What this method does]
    ///
    /// # Errors
    ///
    /// [If applicable: Implementations may return an error when...]
    fn required_method(&self) -> ReturnType;

    /// [What this default method does]
    fn provided_method(&self) {
        // default implementation
    }
}
```

**Trait Documentation Rules:**

1. **Summary Line**: What behavior does this trait enable?
   - Good: "Types that can be serialized to JSON."
   - Bad: "A trait for serialization."

2. **Contract Documentation**:
   - Explain what implementors must guarantee
   - Document invariants
   - Explain relationship between methods (if any)

3. **Examples Section** (REQUIRED):
   - Show a complete implementation
   - Include all required methods
   - Show the trait in use

### 2. Document Required Methods

For each required method in the trait:

```rust
/// [One-line summary in third-person singular]
///
/// [Detailed explanation]
///
/// # Panics
///
/// [Implementations may panic if...]
///
/// # Errors
///
/// [Implementations may return an error when...]
fn method_name(&self, arg: Type) -> ReturnType;
```

**Trait Method Documentation Rules:**

1. **Document the contract**: What must implementors ensure?
2. **Error conditions**: When may implementations return errors?
3. **Panic conditions**: When may implementations panic?
4. **Default behavior**: What happens if not overridden?

### 3. Document Provided Methods

For methods with default implementations:

```rust
/// [One-line summary]
///
/// [Explanation of default behavior]
///
/// # Examples
///
/// ```
/// // Using default implementation
/// impl TraitName for MyType {}
///
/// // Overriding default
/// impl TraitName for MyType {
///     fn provided_method(&self) {
///         // custom implementation
///     }
/// }
/// ```
fn provided_method(&self) {
    // default implementation
}
```

### 4. Document Associated Types

If the trait has associated types:

```rust
/// Types that can be iterated over.
pub trait Iterable {
    /// The type of items yielded by the iterator.
    type Item;

    /// Returns an iterator over the items.
    fn iter(&self) -> impl Iterator<Item = Self::Item>;
}
```

### 5. Document Associated Constants

If the trait has associated constants:

```rust
/// Types with a maximum size limit.
pub trait Bounded {
    /// The maximum number of elements this type can hold.
    const MAX_SIZE: usize;
}
```

### 6. Document Supertraits

If the trait extends other traits:

```rust
/// Types that can be displayed and serialized.
///
/// This trait requires implementors to also implement [`Display`] and [`Serialize`].
///
/// [`Display`]: trait.Display.html
/// [`Serialize`]: trait.Serialize.html
pub trait DisplaySerializable: Display + Serialize {
    // ...
}
```

### 7. Document Auto Traits (if applicable)

For unsafe auto traits:

```rust
/// A marker trait for types that are safe to share between threads.
///
/// # Safety
///
/// Implementing this trait promises that the type is thread-safe.
/// This means:
/// - No interior mutability without synchronization
/// - No thread-local data
/// - No data races possible
pub unsafe auto trait Send {}
```

### 8. Cross-Reference Related Traits

Use intra-doc links:

```rust
/// For mutable iteration, see [`IntoIterator`].
///
/// [`IntoIterator`]: trait.IntoIterator.html
```

### 9. Present Generated Documentation

Show {user_name} the trait documentation:

"**Trait Documentation Generated**

**Trait: [TraitName]**
```rust
[SHOW /// DOCUMENTATION AND TRAIT DEFINITION]
```

**RFC 1574 Compliance Check:**
- ✅ Outer doc comments (`///`)
- ✅ Summary line: what behavior is enabled
- ✅ Contract documented for implementors
- ✅ All required methods documented
- ✅ All provided methods documented
- ✅ Examples section with complete implementation
- ✅ Panics/Errors documented for trait methods
- ✅ Intra-doc links for related traits

Review trait documentation. Any adjustments needed?"

### 10. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C:
  - Update {outputFile}: append trait documentation under "## Trait Documentation" section
  - Update frontmatter: stepsCompleted: step-07-document-traits
  - Then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#10-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All traits documented with `///`
- Summary lines explain enabled behavior
- Contracts documented for implementors
- All trait methods documented
- Examples show complete implementations
- RFC 1574 compliant
- Output file updated

### ❌ SYSTEM FAILURE:

- Missing trait method documentation
- No contract explanation
- Missing examples
- Not RFC 1574 compliant
