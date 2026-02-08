---
name: 'step-06-document-functions'
description: 'Generate function and method documentation'
nextStepFile: './step-07-document-traits.md'
---

# Step 6: Document Functions and Methods

## STEP GOAL:

Generate RFC 1574 compliant documentation for all functions, methods, and unsafe functions using outer doc comments (`///`).

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Use outer doc comments `///` for functions

### Role Reinforcement:

- ✅ You are a rustdoc specialist creating function documentation
- ✅ Third-person singular present indicative: "Returns", "Converts"

### Step-Specific Rules:

- 🎯 Summary line: third-person singular ("Returns", not "Return")
- 🎯 NEVER document the obvious (don't repeat type signature)
- 🎯 ALWAYS include Examples section
- 🎯 Include Panics/Errors/Safety as needed

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📝 Generate function docs in proper format
- 💾 Append to output file

## CONTEXT BOUNDARIES:

- Available context: Analysis from Step 2, type docs complete
- Focus: Function/method documentation
- Limits: Include all public functions and methods

## MANDATORY SEQUENCE

### 1. Document Functions

For each function, follow this format:

```rust
/// [One-line summary in third-person singular]
///
/// [Detailed explanation if behavior is non-obvious]
///
/// # Panics
///
/// [If applicable: Panics if `index` is out of bounds.]
///
/// # Errors
///
/// [If returns Result: Returns an error when...]
///
/// # Safety
///
/// [If unsafe: Caller must ensure...]
///
/// # Examples
///
/// ```
/// use [crate]::[module]::[function_name];
///
/// let result = function_name([args]);
/// assert_eq!(result, [expected]);
/// ```
pub fn function_name(arg: Type) -> ReturnType {
    // ...
}
```

**Function Documentation Rules:**

1. **Summary Line** (REQUIRED):
   - Use third-person singular present indicative
   - Good: "Returns the number of elements in the vector."
   - Bad: "Return the number..." or "This function returns..."

2. **AVOID Anti-Patterns**:
   ```rust
   // BAD - repeats type signature:
   /// Parameters:
   /// - `a`: an immutable reference to a BoundingBox

   // GOOD - explains semantics:
   /// Returns a new [`BoundingBox`] that exactly encompasses both inputs.
   ```

3. **Examples Section** (REQUIRED):
   ```rust
   /// # Examples
   ///
   /// ```
   /// use std::env;
   ///
   /// for argument in env::args() {
   ///     println!("{argument}");
   /// }
   /// ```
   ```

   For fallible functions, use proper error handling:
   ```rust
   /// # Examples
   ///
   /// ```
   /// # use std::error::Error;
   /// # fn main() -> Result<(), Box<dyn Error>> {
   /// let parsed = "42".parse::<i32>()?;
   /// # Ok(())
   /// # }
   /// ```
   ```

4. **Panics Section** (if applicable):
   ```rust
   /// # Panics
   ///
   /// Panics if `index` is out of bounds.
   ```
   - Document known panic conditions
   - Don't document panics in caller-provided logic (excessive)

5. **Errors Section** (for Result return types):
   ```rust
   /// # Errors
   ///
   /// Returns an error if the file does not exist or cannot be opened.
   ```

6. **Safety Section** (REQUIRED for unsafe functions):
   ```rust
   /// # Safety
   ///
   /// Caller must ensure:
   /// - `ptr` is properly aligned
   /// - `ptr` points to valid memory
   /// - The memory is not accessed elsewhere
   ///
   /// Violating these invariants causes undefined behavior.
   ```

### 2. Document Methods

For methods in `impl` blocks, follow the same rules as functions:

```rust
/// [One-line summary in third-person singular]
///
/// [Detailed explanation]
///
/// # Examples
///
/// ```
/// use [crate]::[Type];
///
/// let instance = Type::new();
/// instance.method_name();
/// ```
pub fn method_name(&self) -> ReturnType {
    // ...
}
```

### 3. Document Constructors (new, etc.)

```rust
/// Creates a new [TypeName] with the given [parameters].
///
/// # Examples
///
/// ```
/// use [crate]::[TypeName];
///
/// let instance = TypeName::new([args]);
/// ```
pub fn new(args: Type) -> Self {
    // ...
}
```

### 4. Document Builder Methods

For builder pattern methods, show the full chain:

```rust
/// Sets the [configuration option].
///
/// # Examples
///
/// ```
/// use [crate]::[Builder];
///
/// let instance = Builder::new()
///     .option_a(value_a)
///     .option_b(value_b)
///     .build();
/// ```
pub fn option_a(self, value: Type) -> Self {
    // ...
}
```

### 5. Cross-Reference Related Functions

Use intra-doc links:

```rust
/// See also [`similar_function`] which [difference].
///
/// [`similar_function`]: fn.similar_function.html
```

### 6. Present Generated Documentation

Show {user_name} the function documentation:

"**Function Documentation Generated**

**Function: [function_name]**
```rust
[SHOW /// DOCUMENTATION AND FUNCTION SIGNATURE]
```

**Method: [method_name]**
```rust
[SHOW /// DOCUMENTATION AND METHOD SIGNATURE]
```

**RFC 1574 Compliance Check:**
- ✅ Outer doc comments (`///`)
- ✅ Summary line: third-person singular
- ✅ No "Parameters:" or "Returns:" sections
- ✅ Examples section (REQUIRED)
- ✅ Panics section (where applicable)
- ✅ Errors section (for Result types)
- ✅ Safety section (for unsafe functions)
- ✅ Intra-doc links for related functions

Review function documentation. Any adjustments needed?"

### 7. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C:
  - Update {outputFile}: append function documentation under "## Function Documentation" section
  - Update frontmatter: stepsCompleted: step-06-document-functions
  - Then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#7-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All functions documented with `///`
- All methods documented with `///`
- Summary lines use third-person singular
- Examples sections present
- Panics/Errors/Safety sections where required
- No anti-patterns (Parameters/Returns sections)
- RFC 1574 compliant
- Output file updated

### ❌ SYSTEM FAILURE:

- Missing Examples section
- Using "Parameters:" or "Returns:" sections
- Not using third-person singular
- Missing Safety section for unsafe functions
- Missing Panics section where applicable
- Not RFC 1574 compliant
