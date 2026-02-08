---
name: 'step-05-document-types'
description: 'Generate struct and enum documentation'
nextStepFile: './step-06-document-functions.md'
---

# Step 5: Document Structs and Enums

## STEP GOAL:

Generate RFC 1574 compliant documentation for all structs and enums using outer doc comments (`///`).

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Use outer doc comments `///` for types

### Role Reinforcement:

- ✅ You are a rustdoc specialist creating type documentation
- ✅ Component-type granularity is critical

### Step-Specific Rules:

- 🎯 MUST use `///` syntax
- 🎯 Document what the type REPRESENTS, not just its fields
- 🎯 Document each field/variant with inline comments
- 🎯 Examples are REQUIRED

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📝 Generate type docs in proper format
- 💾 Append to output file

## CONTEXT BOUNDARIES:

- Available context: Analysis from Step 2, crate/module docs complete
- Focus: Type-level documentation (structs, enums)
- Limits: Separate from function/trait docs

## MANDATORY SEQUENCE

### 1. Document Structs

For each struct, follow this format:

```rust
/// [One-line summary: what this struct represents]
///
/// [More detailed explanation - when to use, key behaviors]
///
/// # Examples
///
/// ```
/// use [crate]::[module]::[StructName];
///
/// let instance = StructName::new([args]);
/// [demonstrate usage]
/// ```
pub struct StructName {
    /// [Description of this field]
    pub field_name: FieldType,
    /// [Description of this field]
    pub another_field: AnotherType,
}
```

**Struct Documentation Rules:**

1. **Summary Line**: What does this struct represent?
   - Good: "A configuration builder for HTTP client settings."
   - Bad: "A struct with configuration fields."

2. **Field Documentation**: Inline `///` before each public field:
   ```rust
   pub struct Person {
       /// The person's full name.
       pub name: String,
       /// Age in years.
       pub age: u8,
   }
   ```

3. **Examples Section (REQUIRED)**:
   - Show construction
   - Show typical usage
   - Use `?` not `unwrap()` for fallible operations

4. **Special Sections (if applicable)**:
   ```rust
   /// # Panics
   ///
   /// [When/why this struct's methods might panic]
   ///
   /// # Errors
   ///
   /// [Error conditions for fallible methods]
   ```

### 2. Document Enums

For each enum, follow this format:

```rust
/// [One-line summary: what this enum represents]
///
/// [When to use this enum vs alternatives]
///
/// # Examples
///
/// ```
/// use [crate]::[module]::[EnumName];
///
/// let value = EnumName::Variant;
///
/// match value {
///     EnumName::First => [handle first],
///     EnumName::Second => [handle second],
/// }
/// ```
pub enum EnumName {
    /// [Description of when this variant is used]
    First,
    /// [Description of when this variant is used]
    ///
    /// The contained data represents [explanation].
    Second(ContainedType),
    /// [Description]
    Third { field: Type },
}
```

**Enum Documentation Rules:**

1. **Summary Line**: What does this enum represent?
   - Good: "Possible errors that can occur during file parsing."
   - Bad: "An enum of error types."

2. **Variant Documentation**: Inline `///` before each variant:
   ```rust
   pub enum Option<T> {
       /// No value.
       None,
       /// Some value of type `T`.
       Some(T),
   }
   ```

3. **Data Variants**: If variant has data, explain what the data represents

4. **Examples Section (REQUIRED)**:
   - Show match patterns
   - Include both common and edge cases
   - Show `None` for Option, `Err` for Result:
   ```rust
   /// `None` will result in [behavior]:
   ///
   /// ```
   /// let x: Option<i32> = None;
   /// [show behavior]
   /// ```
   ```

### 3. Document Type Aliases

```rust
/// [One-line description]
///
/// # Examples
///
/// ```
/// use [crate]::[TypeAlias];
///
/// let instance: TypeAlias = [value];
/// ```
pub type TypeAlias = OriginalType;
```

### 4. Document Constants

```rust
/// [Description of what this constant represents]
///
/// # Examples
///
/// ```
/// use [crate]::[CONST_NAME];
///
/// assert_eq!(CONST_NAME, [expected_value]);
/// ```
pub const CONST_NAME: Type = value;
```

### 5. Cross-Reference Types

Use intra-doc links for related types:

```rust
/// See also [`OtherStruct`] for [related functionality].
///
/// [`OtherStruct`]: struct.OtherStruct.html
```

### 6. Present Generated Documentation

Show {user_name} the type documentation:

"**Type Documentation Generated**

**Struct: [StructName]**
```rust
[SHOW /// DOCUMENTATION AND STRUCT DEFINITION]
```

**Enum: [EnumName]**
```rust
[SHOW /// DOCUMENTATION AND ENUM DEFINITION]
```

**RFC 1574 Compliance Check:**
- ✅ Outer doc comments (`///`)
- ✅ Summary line first for each type
- ✅ All public fields documented
- ✅ All variants documented
- ✅ Examples section (REQUIRED)
- ✅ Edge cases shown for enums
- ✅ Intra-doc links for related types

Review type documentation. Any adjustments needed?"

### 7. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C:
  - Update {outputFile}: append type documentation under "## Type Documentation" section
  - Update frontmatter: stepsCompleted: step-05-document-types
  - Then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#7-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All structs documented with `///`
- All enums documented with `///`
- All public fields have inline docs
- All variants have inline docs
- Examples sections present
- RFC 1574 compliant
- Output file updated

### ❌ SYSTEM FAILURE:

- Missing field documentation
- Missing variant documentation
- No examples sections
- Not RFC 1574 compliant
- Missing edge case examples for enums
