# HashMap Refactoring Plan - FINAL
## Complete Implementation Guide with Context & Test Strategy

**Status**: Ready for implementation
**Estimated Duration**: ~4-5 hours
**Return Point**: Phase 5.2 of loader-ingestor refactoring

---

## Table of Contents
1. [Problem Analysis & Context](#problem-analysis--context)
2. [Architectural Decision](#architectural-decision)
3. [Detailed Design](#detailed-design)
4. [Test Strategy](#test-strategy)
5. [Implementation Plan](#implementation-plan)
6. [Verification & Rollback](#verification--rollback)

---

## Problem Analysis & Context

### The Original Question
**"Why use BTreeMap over HashMap for properties?"**

Investigation revealed:
1. **Deterministic ordering was assumed necessary** for correctness
2. **Two-pointer merge algorithm** in Resolver required sorted Vec
3. **No actual requirement** for deterministic serialization (staleness uses file hashes, not byte comparison)
4. **Performance cost**: O(n) property lookups hurt LSP performance

### Current State Analysis

#### Collection Types in Use
```rust
// Domain aggregates
Schema.properties: Vec<Property>                           // O(n) lookup ❌
PropertyBank.properties: BTreeMap<PropertyName, Property>  // O(log n) lookup ⚠️

// Processing intermediates
RefExpandedSchema.properties: Vec<Property>                // Sorted for merge
SchemaNode.properties: Vec<Property>                       // Sorted for merge

// Storage
SchemaVersion.expanded_properties: HashMap<PropertyName, Property> // ✅ Already correct!
HashMetadata.properties: BTreeMap<PropertyName, [u8; 32]> // Used for comparison
```

#### Why Sorted Vec Was Used

**Resolver.merge_properties** (lines 167-215):
```rust
fn merge_properties(
    parent: &[Property],
    own: &[Property],
    excludes: &[Box<str>],
) -> Vec<Property> {
    // 50+ lines of two-pointer sorted merge
    // REQUIRES both inputs sorted by property name
    let mut p_iter = parent.iter().peekable();
    let mut c_iter = own.iter().peekable();

    loop {
        match (p_iter.peek(), c_iter.peek()) {
            (Some(&p), Some(&c)) => {
                match p.name().as_str().cmp(c.name().as_str()) {
                    Ordering::Less => { /* parent first */ }
                    Ordering::Greater => { /* child first */ }
                    Ordering::Equal => { /* child overrides */ }
                }
            }
            // ...
        }
    }
}
```

**This optimization is NOT worth the O(n) lookup penalty for LSP operations.**

### Future Requirements

**LSP Hot Paths** (will be called frequently):
```rust
// Autocomplete: "Show all properties for schema X"
schema.properties()  // O(1) iteration (both Vec and HashMap work)

// Hover: "Show details for property 'title'"
schema.find_property_by_name("title")  // O(n) with Vec, O(1) with HashMap ⚠️

// Validation: "Does schema X have property Y?"
schema.has_property("author")  // O(n) with Vec, O(1) with HashMap ⚠️

// Property details by ID (from note frontmatter)
schema.find_property_by_id(prop_id)  // O(n) for both (acceptable - rare operation)
```

**Performance Requirements**:
- Hover: < 50ms response time
- Autocomplete: < 100ms
- Validation: < 200ms for entire note

With 1,000 properties per schema:
- Vec: 1,000 comparisons = ~10ms per lookup
- HashMap: ~1 lookup = ~0.01ms per lookup

### The Core Issue: Naming Confusion

**Current "Resolver"** actually does **schema merging** (combines property HashMaps):
```rust
impl Resolver {
    pub fn resolve(tree: &SchemaTree, ...) -> Result<Vec<Schema>, ...> {
        // Walks tree, merges parent + child properties
    }

    fn merge_properties(...) -> Vec<Property> {
        // Two-pointer merge algorithm
    }
}
```

**Expander** duplicates property override logic:
```rust
impl RefExpander {
    fn apply_ref_overrides(...) -> Result<Property, ...> {
        // Overrides optionality, multiplicity, spec
    }

    fn apply_spec_overrides(...) -> Result<PropertySpec, ...> {
        // Type-specific constraint overrides
        // Type safety checks (bool → number rejected)
    }
}
```

**Problem**: Property-level override logic is duplicated and embedded in two different components.

---

## Architectural Decision

### New Architecture

```
PropertyResolver (NEW - in resolver.rs)
├─ Property-level conflict resolution
├─ Used by BOTH Expander and Merger
└─ Methods:
   ├─ resolve_optionality()     // Override required/optional
   ├─ resolve_multiplicity()    // Override single/multi
   ├─ resolve_spec()            // Type-specific overrides (moved from Expander)
   ├─ resolve_from_bank_ref()   // For Expander ($ref resolution)
   └─ resolve_child_override()  // For Merger (inheritance)

Merger (RENAMED from Resolver - in merger.rs)
├─ Schema-level property merging
├─ Uses PropertyResolver for conflicts
└─ Methods:
   ├─ resolve()                 // Walk tree, merge schemas
   ├─ merge_properties()        // Combine HashMaps (simplified!)
   └─ is_excluded()            // Helper

Expander (UPDATED - in expander.rs)
├─ Uses PropertyResolver instead of local methods
└─ Methods:
   ├─ expand_schema()           // Builds HashMap
   └─ expand_property()         // Uses PropertyResolver

Extender (UNCHANGED - in extender.rs)
└─ Builds inheritance tree (no property logic)
```

### Naming Rationale

| Component | Responsibility | Level | Input | Output |
|-----------|---------------|-------|-------|--------|
| **PropertyResolver** | Resolves property conflicts | Single property | 2 Properties / Property + overrides | 1 Property |
| **Merger** | Merges schema properties | Schema | 2 HashMaps + excludes | 1 HashMap |
| **Expander** | Expands $ref to properties | Schema | RawSchema + Bank | HashMap\<Name, Property\> |
| **Extender** | Builds inheritance tree | Tree | Vec\<Schemas\> | SchemaTree |

**Why "PropertyResolver" not "PropertyMerger"?**
- "Resolve" = handle conflicts, make decisions
- "Merge" = combine collections
- PropertyResolver resolves conflicts between individual properties
- Merger merges collections of properties

---

## Detailed Design

### 1. PropertyResolver (NEW)

**File**: `lithos-core/src/schema/resolver.rs` (REPURPOSED)

```rust
//! Property-level conflict resolution and override logic.
//!
//! Handles resolving conflicts between property definitions and applying
//! overrides while maintaining type safety.
//!
//! ## Use Cases
//!
//! ### Expander: PropertyBank Reference Overrides
//! ```ignore
//! // Schema file has: { "$ref": "#property_bank/title", "required": false }
//! // Bank has:        Property { name: "title", required: true, ... }
//! // Result:          Property { name: "title", required: false, ... }
//! ```
//!
//! ### Merger: Schema Inheritance Overrides
//! ```ignore
//! // Parent schema:  Property { name: "title", required: true, ... }
//! // Child schema:   Property { name: "title", required: false, ... }
//! // Result:         Property { name: "title", required: false, ... }
//! ```

use super::{
    error::SchemaError,
    property::{Multiplicity, Optionality, Property},
    property_spec::PropertySpec,
    raw::property::RawPropertyRef,
};

/// Resolves property-level conflicts and applies overrides.
///
/// Stateless utility for property override logic, ensuring type safety
/// and validation when combining property definitions from different sources.
pub struct PropertyResolver;

impl PropertyResolver {
    /// Resolve optionality override.
    ///
    /// # Rules
    /// - If override is `Some`, use it
    /// - Otherwise, use base optionality
    ///
    /// # Examples
    /// ```ignore
    /// // Override required → optional
    /// let base = Optionality::Required;
    /// let override_val = Some(false);
    /// let result = PropertyResolver::resolve_optionality(base, override_val);
    /// assert_eq!(result, Optionality::Optional);
    ///
    /// // No override → keep base
    /// let result = PropertyResolver::resolve_optionality(base, None);
    /// assert_eq!(result, Optionality::Required);
    /// ```
    #[inline]
    pub fn resolve_optionality(
        base: Optionality,
        override_required: Option<bool>,
    ) -> Optionality {
        override_required.map_or(base, Optionality::from)
    }

    /// Resolve multiplicity override.
    ///
    /// # Rules
    /// - If override is `Some`, use it
    /// - Otherwise, use base multiplicity
    #[inline]
    pub fn resolve_multiplicity(
        base: Multiplicity,
        override_multi: Option<bool>,
    ) -> Multiplicity {
        override_multi.map_or(base, Multiplicity::from)
    }

    /// Resolve property spec overrides (type-specific constraints).
    ///
    /// # Rules
    /// - Cannot change property type (bool → number rejected)
    /// - Can override type-specific constraints (min/max, pattern, etc.)
    ///
    /// # Errors
    /// Returns `SchemaError::PropertyTypeMismatch` if override attempts
    /// to change the property type.
    ///
    /// # Examples
    /// ```ignore
    /// // Valid: Override number constraints
    /// let base = PropertySpec::Number(NumberSpec { min: None, max: None });
    /// let overrides = RawPropertyRef { number: RawNumberSpec { min: Some(0.0), .. }, .. };
    /// let result = PropertyResolver::resolve_spec(&base, &overrides)?;
    /// // Result: NumberSpec { min: Some(0.0), max: None }
    ///
    /// // Invalid: Attempt to change type
    /// let base = PropertySpec::Bool(BoolSpec);
    /// let overrides = RawPropertyRef { number: RawNumberSpec { min: Some(0.0), .. }, .. };
    /// let result = PropertyResolver::resolve_spec(&base, &overrides);
    /// // Result: Err(PropertyTypeMismatch { expected: "bool", actual: "number" })
    /// ```
    pub fn resolve_spec(
        base: &PropertySpec,
        ref_entry: &RawPropertyRef,
    ) -> Result<PropertySpec, SchemaError> {
        // Detect which type-specific overrides are present
        let has_number = ref_entry.number.min.is_some()
            || ref_entry.number.max.is_some()
            || ref_entry.number.step.is_some();
        let has_string = ref_entry.string.options.is_some()
            || ref_entry.string.pattern.is_some();
        let has_date = ref_entry.date.format.is_some();
        let has_file = ref_entry.file.directory.is_some()
            || ref_entry.file.file_class.is_some();

        match base {
            PropertySpec::Bool(_) => {
                if has_number {
                    return Err(type_mismatch("bool", "number"));
                }
                if has_string {
                    return Err(type_mismatch("bool", "string"));
                }
                if has_date {
                    return Err(type_mismatch("bool", "date"));
                }
                if has_file {
                    return Err(type_mismatch("bool", "file"));
                }
                Ok(base.clone())
            }

            PropertySpec::Number(spec) => {
                if has_string {
                    return Err(type_mismatch("number", "string"));
                }
                if has_date {
                    return Err(type_mismatch("number", "date"));
                }
                if has_file {
                    return Err(type_mismatch("number", "file"));
                }
                Ok(PropertySpec::Number(
                    spec.clone().apply_overrides(&ref_entry.number)?,
                ))
            }

            PropertySpec::String(spec) => {
                if has_number {
                    return Err(type_mismatch("string", "number"));
                }
                if has_date {
                    return Err(type_mismatch("string", "date"));
                }
                if has_file {
                    return Err(type_mismatch("string", "file"));
                }
                Ok(PropertySpec::String(
                    spec.clone().apply_overrides(&ref_entry.string)?,
                ))
            }

            PropertySpec::Date(spec) => {
                if has_number {
                    return Err(type_mismatch("date", "number"));
                }
                if has_string {
                    return Err(type_mismatch("date", "string"));
                }
                if has_file {
                    return Err(type_mismatch("date", "file"));
                }
                Ok(PropertySpec::Date(
                    spec.clone().apply_overrides(&ref_entry.date)?,
                ))
            }

            PropertySpec::File(spec) => {
                if has_number {
                    return Err(type_mismatch("file", "number"));
                }
                if has_string {
                    return Err(type_mismatch("file", "string"));
                }
                if has_date {
                    return Err(type_mismatch("file", "date"));
                }
                Ok(PropertySpec::File(
                    spec.clone().apply_overrides(&ref_entry.file)?,
                ))
            }
        }
    }

    /// Apply all overrides from a property bank reference.
    ///
    /// Used by Expander when resolving `$ref` entries.
    ///
    /// # Errors
    /// Returns error if type mismatch occurs during spec override.
    pub fn resolve_from_bank_ref(
        bank_property: &Property,
        ref_entry: &RawPropertyRef,
    ) -> Result<Property, SchemaError> {
        let optionality = Self::resolve_optionality(
            bank_property.optionality(),
            ref_entry.required,
        );
        let multiplicity = Self::resolve_multiplicity(
            bank_property.multiplicity(),
            ref_entry.multi,
        );
        let spec = Self::resolve_spec(bank_property.spec(), ref_entry)?;

        Ok(Property::new(
            bank_property.id(),
            bank_property.name().clone(),
            optionality,
            multiplicity,
            spec,
        ))
    }

    /// Apply child property override to parent property.
    ///
    /// Used by Merger during schema inheritance. In schema inheritance,
    /// child property completely replaces parent property (no field merging).
    ///
    /// # Rules
    /// - Child property wins entirely
    /// - Child can change optionality, multiplicity, spec, and type
    /// - Child's PropertyId is used (new property instance)
    ///
    /// # Examples
    /// ```ignore
    /// // Parent: title (required, single, String[max=100])
    /// // Child:  title (optional, multi, String[max=200])
    /// // Result: title (optional, multi, String[max=200]) - child wins completely
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve_child_override(
        _parent: &Property,
        child: &Property,
    ) -> Property {
        // In schema inheritance, child completely replaces parent
        // No merging of fields - child wins entirely
        child.clone()
    }
}

#[inline]
fn type_mismatch(expected: &str, actual: &str) -> SchemaError {
    SchemaError::PropertyTypeMismatch {
        expected: expected.into(),
        actual: actual.into(),
    }
}
```

### 2. Merger (RENAMED from Resolver)

**File**: `lithos-core/src/schema/merger.rs` (NEW - renamed from resolver.rs)

```rust
//! Schema-level property merging for inheritance.
//!
//! Combines properties from parent and child schemas following inheritance rules:
//! - Child properties override parent properties with same name
//! - Parent properties in excludes list are filtered out
//! - All other parent properties are inherited
//!
//! Uses `PropertyResolver` for individual property conflict resolution.

use std::collections::HashMap;

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    error::SchemaError,
    extender::SchemaTree,
    property::{Property, PropertyName},
    resolver::PropertyResolver,
};

/// Merges schema properties following inheritance rules.
///
/// Handles schema-level merging by combining property HashMaps from
/// parent and child schemas, with PropertyResolver handling conflicts.
pub struct Merger;

impl Merger {
    /// Resolve all schemas in tree using topological order.
    ///
    /// Walks the tree from roots to leaves, merging each schema with its
    /// parent's properties.
    ///
    /// # Arguments
    /// * `tree` - Topologically sorted inheritance tree
    /// * `known_parents` - DB-fresh parent schemas (not in tree)
    ///
    /// # Returns
    /// Fully resolved schemas with inherited properties
    ///
    /// # Errors
    /// Returns error if schema name validation fails
    pub fn resolve(
        tree: &SchemaTree,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<Vec<Schema>, SchemaError> {
        let mut resolved_cache: HashMap<SchemaId, Schema> = HashMap::new();
        let mut results = Vec::with_capacity(tree.nodes().len());

        for &id in tree.nodes() {
            let node = tree
                .node(id)
                .expect("topological order contains only valid IDs");

            // Get parent properties (from cache or known_parents)
            let parent_props: &HashMap<PropertyName, Property> =
                if let Some(parent_id) = node.parent_id {
                    resolved_cache
                        .get(&parent_id)
                        .or_else(|| known_parents.get(&parent_id))
                        .map(Schema::properties)
                        .unwrap_or_else(|| {
                            tracing::warn!(
                                schema_id = ?id,
                                parent_id = ?parent_id,
                                "Parent schema not found in cache or known_parents"
                            );
                            &EMPTY_PROPERTIES
                        })
                } else {
                    &EMPTY_PROPERTIES
                };

            // Merge parent and child properties
            let merged = Self::merge_properties(
                parent_props,
                &node.properties,
                &node.excludes,
            );

            let schema_name = SchemaName::try_new(&node.name)?;
            let schema = Schema::new(
                id,
                schema_name,
                node.parent_id,
                node.children.clone(),
                merged,
            );

            resolved_cache.insert(id, schema.clone());
            results.push(schema);
        }

        Ok(results)
    }

    /// Merge parent and child properties with exclusion and override rules.
    ///
    /// # Rules
    /// 1. Parent properties included unless:
    ///    - Name is in excludes list
    ///    - Child has property with same name (override)
    /// 2. All child properties included
    /// 3. Property conflicts resolved via `PropertyResolver::resolve_child_override`
    ///
    /// # Performance
    /// - Time: O(p + c) where p=parent props, c=child props
    /// - Space: O(p + c) for result HashMap
    ///
    /// # Examples
    /// ```ignore
    /// let parent = HashMap::from([
    ///     (name_a, prop_a_v1),  // Will be overridden
    ///     (name_b, prop_b),     // Will be excluded
    ///     (name_c, prop_c),     // Will be inherited
    /// ]);
    /// let child = HashMap::from([
    ///     (name_a, prop_a_v2),  // Overrides parent
    ///     (name_d, prop_d),     // New property
    /// ]);
    /// let excludes = vec!["b".into()];
    ///
    /// let result = Merger::merge_properties(&parent, &child, &excludes);
    /// // result contains: prop_a_v2 (override), prop_c (inherited), prop_d (new)
    /// // result does NOT contain: prop_b (excluded)
    /// ```
    fn merge_properties(
        parent: &HashMap<PropertyName, Property>,
        child: &HashMap<PropertyName, Property>,
        excludes: &[Box<str>],
    ) -> HashMap<PropertyName, Property> {
        let mut result = HashMap::with_capacity(parent.len() + child.len());

        // Step 1: Add parent properties (filter excluded and overridden)
        for (name, prop) in parent {
            if !Self::is_excluded(name, excludes) && !child.contains_key(name) {
                result.insert(name.clone(), prop.clone());
            }
        }

        // Step 2: Add child properties (resolve conflicts with PropertyResolver)
        for (name, child_prop) in child {
            let resolved = if let Some(parent_prop) = parent.get(name) {
                // Child overrides parent - use PropertyResolver
                PropertyResolver::resolve_child_override(parent_prop, child_prop)
            } else {
                // New property - just use child
                child_prop.clone()
            };
            result.insert(name.clone(), resolved);
        }

        result
    }

    /// Check if property name appears in excludes list.
    ///
    /// Case-sensitive string comparison.
    #[inline]
    fn is_excluded(name: &PropertyName, excludes: &[Box<str>]) -> bool {
        excludes.iter().any(|e| e.as_ref() == name.as_str())
    }
}

// Empty properties constant to avoid repeated allocations
static EMPTY_PROPERTIES: HashMap<PropertyName, Property> = HashMap::new();

// NOTE: incremental_resolve_affected_properties moved from old Resolver
// (implementation unchanged - handles PropertyBank cascade updates)
```

### 3. Update Expander

**File**: `lithos-core/src/schema/expander.rs`

```rust
// Update imports:
use super::{
    // ... existing imports
    resolver::PropertyResolver,  // NEW
};

impl<'bank> RefExpander<'bank> {
    fn expand_schema(&self, raw: RawSchema) -> Result<RefExpandedSchema, SchemaError> {
        // Build HashMap instead of sorted Vec
        let mut properties = HashMap::with_capacity(raw.properties.len());

        for (prop_name, entry) in raw.properties {
            let property = self.expand_property(&prop_name, entry)?;
            properties.insert(*property.name(), property);
        }

        Ok(RefExpandedSchema {
            name: raw.name,
            extends: raw.extends,
            excludes: raw.excludes,
            properties,
        })
    }

    fn expand_property(&self, name: &str, entry: RawProperty) -> Result<Property, SchemaError> {
        match entry {
            RawProperty::Inline(inline) => {
                // Unchanged - inline properties don't use resolver
                let prop_name = PropertyName::try_new(name)?;
                let spec = inline.spec.try_into()?;
                let optionality = Optionality::from(inline.required);
                let multiplicity = Multiplicity::from(inline.multi);
                Ok(Property::new(
                    PropertyId::new(),
                    prop_name,
                    optionality,
                    multiplicity,
                    spec,
                ))
            }

            RawProperty::Ref(ref_entry) => {
                // Use PropertyResolver instead of local method
                let prop_ref = BankPropertyRef::try_from(ref_entry.ref_path.as_ref())?;
                let bank_name = prop_ref.name();

                let base = self.bank.get(bank_name).ok_or_else(|| {
                    SchemaError::PropertyRefNotFound(ref_entry.ref_path.to_string())
                })?;

                PropertyResolver::resolve_from_bank_ref(base, &ref_entry)
            }
        }
    }

    // DELETE these methods (moved to PropertyResolver):
    // - apply_ref_overrides
    // - apply_spec_overrides
}
```

---

## Test Strategy

### Existing Test Structure

**Expander** (lithos-core/src/schema/expander.rs):
```
#[cfg(test)]
mod tests {
    mod fixtures { ... }  // Test data builders

    mod expand_property {
        #[test] fn inline_bool_resolves_correctly() { ... }
        #[test] fn ref_resolves_from_bank() { ... }
        #[test] fn ref_overrides_optionality_and_multiplicity() { ... }
        #[test] fn ref_type_mismatch_returns_error() { ... }
    }

    mod expand_all {
        #[test] fn properties_sorted_by_name() { ... }  // DELETE - no longer sorted
        #[test] fn multiple_schemas_expand() { ... }
    }
}
```

**Resolver** (lithos-core/src/schema/resolver.rs):
```
#[cfg(test)]
mod tests {
    mod fixtures { ... }  // Test data builders

    mod resolve {
        #[test] fn root_schema_without_parent() { ... }
        #[test] fn child_inherits_parent_properties() { ... }
        #[test] fn child_overrides_parent_property() { ... }
        #[test] fn child_excludes_parent_property() { ... }
        #[test] fn db_fresh_parent_properties_inherited() { ... }
        #[test] fn inheritance_max_depth_constant_value() { ... }
        #[test] fn inheritance_depth_error_constructs() { ... }
        #[test] fn inheritance_depth_limit_exceeded() { ... }
    }
}
```

### New Test Modules

#### PropertyResolver Tests

**File**: `lithos-core/src/schema/resolver.rs` (new test module)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        property::{Multiplicity, Optionality, PropertyId, PropertyName},
        property_spec::{BoolSpec, NumberSpec, PropertySpec, StringSpec},
        raw::property::{RawPropertyRef, RawNumberSpec, RawStringSpec},
    };

    // ── Fixtures ────────────────────────────────────────────────────────────

    mod fixtures {
        use super::*;

        pub fn bool_property(name: &str) -> Property {
            Property::new(
                PropertyId::new(),
                PropertyName::try_new(name).unwrap(),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec),
            )
        }

        pub fn number_property(name: &str, min: Option<f64>, max: Option<f64>) -> Property {
            Property::new(
                PropertyId::new(),
                PropertyName::try_new(name).unwrap(),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Number(NumberSpec { min, max, step: None }),
            )
        }

        pub fn ref_entry(ref_path: &str) -> RawPropertyRef {
            RawPropertyRef {
                ref_path: ref_path.into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: Default::default(),
                file: Default::default(),
            }
        }

        pub fn ref_with_overrides(
            ref_path: &str,
            required: Option<bool>,
            multi: Option<bool>,
        ) -> RawPropertyRef {
            RawPropertyRef {
                ref_path: ref_path.into(),
                required,
                multi,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: Default::default(),
                file: Default::default(),
            }
        }
    }

    // ── Optionality Resolution ──────────────────────────────────────────────

    mod resolve_optionality {
        use super::*;

        #[test]
        fn uses_override_when_present() {
            let result = PropertyResolver::resolve_optionality(
                Optionality::Required,
                Some(false),
            );
            assert_eq!(result, Optionality::Optional);
        }

        #[test]
        fn uses_base_when_no_override() {
            let result = PropertyResolver::resolve_optionality(
                Optionality::Optional,
                None,
            );
            assert_eq!(result, Optionality::Optional);
        }

        #[test]
        fn can_make_required_optional() {
            let result = PropertyResolver::resolve_optionality(
                Optionality::Required,
                Some(false),
            );
            assert_eq!(result, Optionality::Optional);
        }

        #[test]
        fn can_make_optional_required() {
            let result = PropertyResolver::resolve_optionality(
                Optionality::Optional,
                Some(true),
            );
            assert_eq!(result, Optionality::Required);
        }
    }

    // ── Multiplicity Resolution ─────────────────────────────────────────────

    mod resolve_multiplicity {
        use super::*;

        #[test]
        fn uses_override_when_present() {
            let result = PropertyResolver::resolve_multiplicity(
                Multiplicity::Single,
                Some(true),
            );
            assert_eq!(result, Multiplicity::Many);
        }

        #[test]
        fn uses_base_when_no_override() {
            let result = PropertyResolver::resolve_multiplicity(
                Multiplicity::Many,
                None,
            );
            assert_eq!(result, Multiplicity::Many);
        }

        #[test]
        fn can_make_single_many() {
            let result = PropertyResolver::resolve_multiplicity(
                Multiplicity::Single,
                Some(true),
            );
            assert_eq!(result, Multiplicity::Many);
        }

        #[test]
        fn can_make_many_single() {
            let result = PropertyResolver::resolve_multiplicity(
                Multiplicity::Many,
                Some(false),
            );
            assert_eq!(result, Multiplicity::Single);
        }
    }

    // ── Spec Resolution (Type Safety) ───────────────────────────────────────

    mod resolve_spec {
        use super::*;

        #[test]
        fn bool_rejects_number_override() {
            let base = PropertySpec::Bool(BoolSpec);
            let ref_entry = RawPropertyRef {
                ref_path: "#property_bank/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec {
                    min: Some(0.0),
                    max: None,
                    step: None,
                },
                string: RawStringSpec::default(),
                date: Default::default(),
                file: Default::default(),
            };

            let result = PropertyResolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                SchemaError::PropertyTypeMismatch { .. }
            ));
        }

        #[test]
        fn bool_rejects_string_override() {
            let base = PropertySpec::Bool(BoolSpec);
            let ref_entry = RawPropertyRef {
                ref_path: "#property_bank/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec {
                    options: Some(vec!["a".into()]),
                    pattern: None,
                },
                date: Default::default(),
                file: Default::default(),
            };

            let result = PropertyResolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_err());
        }

        #[test]
        fn number_accepts_number_override() {
            let base = PropertySpec::Number(NumberSpec {
                min: None,
                max: None,
                step: None,
            });
            let ref_entry = RawPropertyRef {
                ref_path: "#property_bank/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec {
                    min: Some(0.0),
                    max: Some(100.0),
                    step: None,
                },
                string: RawStringSpec::default(),
                date: Default::default(),
                file: Default::default(),
            };

            let result = PropertyResolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_ok());
            match result.unwrap() {
                PropertySpec::Number(spec) => {
                    assert_eq!(spec.min, Some(0.0));
                    assert_eq!(spec.max, Some(100.0));
                }
                _ => panic!("Expected Number spec"),
            }
        }

        #[test]
        fn number_rejects_string_override() {
            let base = PropertySpec::Number(NumberSpec {
                min: None,
                max: None,
                step: None,
            });
            let ref_entry = RawPropertyRef {
                ref_path: "#property_bank/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec {
                    options: Some(vec!["a".into()]),
                    pattern: None,
                },
                date: Default::default(),
                file: Default::default(),
            };

            let result = PropertyResolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_err());
        }

        #[test]
        fn string_accepts_string_override() {
            let base = PropertySpec::String(StringSpec {
                options: None,
                pattern: None,
            });
            let ref_entry = RawPropertyRef {
                ref_path: "#property_bank/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec {
                    options: Some(vec!["valid".into()]),
                    pattern: None,
                },
                date: Default::default(),
                file: Default::default(),
            };

            let result = PropertyResolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_ok());
            match result.unwrap() {
                PropertySpec::String(spec) => {
                    assert!(spec.options.is_some());
                }
                _ => panic!("Expected String spec"),
            }
        }
    }

    // ── Full Property Resolution (Bank Ref) ─────────────────────────────────

    mod resolve_from_bank_ref {
        use super::*;

        #[test]
        fn preserves_base_when_no_overrides() {
            let base = fixtures::bool_property("test");
            let ref_entry = fixtures::ref_entry("#property_bank/test");

            let result = PropertyResolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.name(), base.name());
            assert_eq!(prop.optionality(), base.optionality());
            assert_eq!(prop.multiplicity(), base.multiplicity());
        }

        #[test]
        fn applies_optionality_override() {
            let base = fixtures::bool_property("test"); // Required by default
            let ref_entry = fixtures::ref_with_overrides(
                "#property_bank/test",
                Some(false), // Override to optional
                None,
            );

            let result = PropertyResolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.optionality(), Optionality::Optional);
        }

        #[test]
        fn applies_multiplicity_override() {
            let base = fixtures::bool_property("test"); // Single by default
            let ref_entry = fixtures::ref_with_overrides(
                "#property_bank/test",
                None,
                Some(true), // Override to multi
            );

            let result = PropertyResolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
        }

        #[test]
        fn applies_all_overrides() {
            let base = fixtures::bool_property("test");
            let ref_entry = fixtures::ref_with_overrides(
                "#property_bank/test",
                Some(false),
                Some(true),
            );

            let result = PropertyResolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.optionality(), Optionality::Optional);
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
        }

        #[test]
        fn rejects_type_mismatch() {
            let base = fixtures::bool_property("test");
            let ref_entry = RawPropertyRef {
                ref_path: "#property_bank/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec {
                    min: Some(0.0),
                    max: None,
                    step: None,
                },
                string: RawStringSpec::default(),
                date: Default::default(),
                file: Default::default(),
            };

            let result = PropertyResolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_err());
        }
    }

    // ── Child Override (Schema Inheritance) ─────────────────────────────────

    mod resolve_child_override {
        use super::*;

        #[test]
        fn child_completely_replaces_parent() {
            let parent = fixtures::bool_property("title");
            let child = fixtures::number_property("title", Some(0.0), Some(100.0));

            let result = PropertyResolver::resolve_child_override(&parent, &child);

            // Child wins completely
            assert_eq!(result.name(), child.name());
            assert_eq!(result.id(), child.id());
            assert!(matches!(result.spec(), PropertySpec::Number(_)));
        }

        #[test]
        fn child_id_is_preserved() {
            let parent = fixtures::bool_property("test");
            let child = fixtures::bool_property("test");

            let result = PropertyResolver::resolve_child_override(&parent, &child);

            // Child's ID is used
            assert_eq!(result.id(), child.id());
            assert_ne!(result.id(), parent.id());
        }
    }
}
```

#### Merger Tests

**File**: `lithos-core/src/schema/merger.rs` (moved from resolver.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // Move all existing resolver tests here
    // Update to use HashMap instead of Vec

    mod fixtures {
        // Move existing fixtures
        // Update to build HashMap<PropertyName, Property>
    }

    mod merge_properties {
        use super::*;

        #[test]
        fn empty_parent_and_child_produces_empty_result() {
            let parent = HashMap::new();
            let child = HashMap::new();
            let excludes = vec![];

            let result = Merger::merge_properties(&parent, &child, &excludes);
            assert_eq!(result.len(), 0);
        }

        #[test]
        fn parent_only_inherits_all() {
            let mut parent = HashMap::new();
            parent.insert(
                PropertyName::try_new("a").unwrap(),
                fixtures::bool_property("a"),
            );
            let child = HashMap::new();

            let result = Merger::merge_properties(&parent, &child, &[]);
            assert_eq!(result.len(), 1);
            assert!(result.contains_key(&PropertyName::try_new("a").unwrap()));
        }

        #[test]
        fn child_only_includes_all() {
            let parent = HashMap::new();
            let mut child = HashMap::new();
            child.insert(
                PropertyName::try_new("b").unwrap(),
                fixtures::bool_property("b"),
            );

            let result = Merger::merge_properties(&parent, &child, &[]);
            assert_eq!(result.len(), 1);
            assert!(result.contains_key(&PropertyName::try_new("b").unwrap()));
        }

        #[test]
        fn no_overlap_combines_both() {
            let mut parent = HashMap::new();
            parent.insert(
                PropertyName::try_new("a").unwrap(),
                fixtures::bool_property("a"),
            );

            let mut child = HashMap::new();
            child.insert(
                PropertyName::try_new("b").unwrap(),
                fixtures::bool_property("b"),
            );

            let result = Merger::merge_properties(&parent, &child, &[]);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn child_overrides_parent_with_same_name() {
            let parent_prop = fixtures::bool_property("shared");
            let child_prop = fixtures::bool_property("shared");

            let mut parent = HashMap::new();
            parent.insert(*parent_prop.name(), parent_prop.clone());

            let mut child = HashMap::new();
            child.insert(*child_prop.name(), child_prop.clone());

            let result = Merger::merge_properties(&parent, &child, &[]);

            assert_eq!(result.len(), 1);
            // Child's PropertyId wins
            assert_eq!(
                result.get(&PropertyName::try_new("shared").unwrap()).unwrap().id(),
                child_prop.id()
            );
        }

        #[test]
        fn excludes_filters_parent_properties() {
            let mut parent = HashMap::new();
            parent.insert(
                PropertyName::try_new("keep").unwrap(),
                fixtures::bool_property("keep"),
            );
            parent.insert(
                PropertyName::try_new("exclude").unwrap(),
                fixtures::bool_property("exclude"),
            );

            let child = HashMap::new();
            let excludes = vec!["exclude".into()];

            let result = Merger::merge_properties(&parent, &child, &excludes);

            assert_eq!(result.len(), 1);
            assert!(result.contains_key(&PropertyName::try_new("keep").unwrap()));
            assert!(!result.contains_key(&PropertyName::try_new("exclude").unwrap()));
        }

        #[test]
        fn excludes_does_not_affect_child_properties() {
            let parent = HashMap::new();
            let mut child = HashMap::new();
            child.insert(
                PropertyName::try_new("excluded").unwrap(),
                fixtures::bool_property("excluded"),
            );
            let excludes = vec!["excluded".into()];

            let result = Merger::merge_properties(&parent, &child, &excludes);

            // Child properties are NEVER excluded
            assert_eq!(result.len(), 1);
            assert!(result.contains_key(&PropertyName::try_new("excluded").unwrap()));
        }
    }

    mod resolve {
        use super::*;

        // Move existing resolver integration tests here
        // - root_schema_without_parent
        // - child_inherits_parent_properties
        // - child_overrides_parent_property
        // - child_excludes_parent_property
        // - db_fresh_parent_properties_inherited
        // - inheritance_depth_limit_exceeded

        // Update assertions to work with HashMap
    }
}
```

### Test Coverage Matrix

| Component | Module | Behavior | Edge Cases |
|-----------|--------|----------|------------|
| **PropertyResolver** | `resolve_optionality` | Override present, Override absent | Both directions (req→opt, opt→req) |
| | `resolve_multiplicity` | Override present, Override absent | Both directions (single→many, many→single) |
| | `resolve_spec` | All type pairs (bool, number, string, date, file) | Type mismatch errors, Valid overrides |
| | `resolve_from_bank_ref` | No overrides, Single override, All overrides | Type mismatch rejection |
| | `resolve_child_override` | Complete replacement | ID preservation, Type changes allowed |
| **Merger** | `merge_properties` | Empty inputs, Parent only, Child only, No overlap | Override, Excludes, Excludes+Override |
| | `is_excluded` | Name in list, Name not in list | Case sensitivity |
| | `resolve` | Root schemas, Simple inheritance, Deep chains | DB-fresh parents, Multiple children |

---

## Implementation Plan

### Phase 0: Create PropertyResolver ⏱️ 1 hour

- [ ] 0.1: Create `lithos-core/src/schema/resolver.rs` (new file)
  - Copy type_mismatch helper from expander.rs
  - Implement resolve_optionality()
  - Implement resolve_multiplicity()
  - Copy and adapt apply_spec_overrides → resolve_spec()
  - Implement resolve_from_bank_ref()
  - Implement resolve_child_override()

- [ ] 0.2: Add comprehensive tests (fixtures + 6 submodules)
  - fixtures module (test data builders)
  - resolve_optionality (4 tests)
  - resolve_multiplicity (4 tests)
  - resolve_spec (6 tests covering all type pairs)
  - resolve_from_bank_ref (5 tests)
  - resolve_child_override (2 tests)

- [ ] 0.3: Run tests: `cargo nextest run -p lithos-core --lib schema::resolver`

### Phase 1: Rename Resolver → Merger ⏱️ 30 min

- [ ] 1.1: Create `lithos-core/src/schema/merger.rs`
  - Copy content from resolver.rs
  - Rename struct: `Resolver` → `Merger`
  - Update all internal references
  - Update doc comments

- [ ] 1.2: Update module exports in `schema/mod.rs`
  ```rust
  pub mod expander;
  pub mod extender;
  pub mod merger;     // NEW (renamed from resolver)
  pub mod resolver;   // NEW (property-level)
  ```

- [ ] 1.3: Update imports across codebase
  - [ ] lithos-core/src/schema/loader.rs
  - [ ] lithos-core/tests/schema_resolution.rs
  - [ ] Any other files importing Resolver

- [ ] 1.4: Run tests: `cargo nextest run --workspace`

### Phase 2: Update Collections to HashMap ⏱️ 45 min

- [ ] 2.1: Update Schema.properties
  **File**: `lithos-core/src/schema/aggregate.rs`
  ```rust
  // Change field:
  properties: HashMap<PropertyName, Property>

  // Update constructor:
  pub fn new(..., properties: HashMap<PropertyName, Property>) -> Self

  // Update accessors:
  pub fn properties(&self) -> &HashMap<PropertyName, Property>

  pub fn find_property_by_name(&self, name: &PropertyName) -> Option<&Property> {
      self.properties.get(name)  // NEW - O(1)
  }

  pub fn find_property(&self, id: &PropertyId) -> Option<&Property> {
      self.properties.values().find(|p| p.id() == *id)
  }
  ```

- [ ] 2.2: Update PropertyBank.properties
  **File**: `lithos-core/src/schema/bank.rs`
  ```rust
  // Change BTreeMap → HashMap
  properties: HashMap<PropertyName, Property>
  ```

- [ ] 2.3: Update RefExpandedSchema.properties
  **File**: `lithos-core/src/schema/expander.rs`
  ```rust
  // Change Vec → HashMap
  pub struct RefExpandedSchema {
      pub properties: HashMap<PropertyName, Property>,
  }
  ```

- [ ] 2.4: Update SchemaNode.properties
  **File**: `lithos-core/src/schema/extender.rs`
  ```rust
  pub(crate) struct SchemaNode {
      pub properties: HashMap<PropertyName, Property>,
  }
  ```

- [ ] 2.5: Update HashMetadata.properties
  **File**: `lithos-core/src/schema/views/metadata.rs`
  ```rust
  properties: HashMap<PropertyName, [u8; 32]>

  // Update return types:
  pub fn compute_property_hashes(...) -> HashMap<PropertyName, [u8; 32]>
  pub fn changed_properties(&self, new_hashes: &HashMap<...>) -> Vec<PropertyName>
  ```

### Phase 3: Refactor Merger ⏱️ 30 min

- [ ] 3.1: Update merge_properties signature and implementation
  **File**: `lithos-core/src/schema/merger.rs`
  ```rust
  fn merge_properties(
      parent: &HashMap<PropertyName, Property>,
      child: &HashMap<PropertyName, Property>,
      excludes: &[Box<str>],
  ) -> HashMap<PropertyName, Property> {
      // New HashMap-based logic (see Detailed Design)
  }
  ```

- [ ] 3.2: Update resolve() to use HashMap
  - Update parent_props type
  - Update merge_properties call
  - Update Schema::new call

- [ ] 3.3: Add PropertyResolver import and use
  ```rust
  use super::resolver::PropertyResolver;

  // In merge_properties:
  let resolved = PropertyResolver::resolve_child_override(parent_prop, child_prop);
  ```

- [ ] 3.4: Delete old methods
  - Remove two-pointer merge implementation
  - Remove push_unless_excluded (inline is_excluded check)

### Phase 4: Refactor Expander ⏱️ 30 min

- [ ] 4.1: Add PropertyResolver import
  ```rust
  use super::resolver::PropertyResolver;
  ```

- [ ] 4.2: Update expand_schema to build HashMap
  ```rust
  fn expand_schema(&self, raw: RawSchema) -> Result<RefExpandedSchema, SchemaError> {
      let mut properties = HashMap::with_capacity(raw.properties.len());
      for (prop_name, entry) in raw.properties {
          let property = self.expand_property(&prop_name, entry)?;
          properties.insert(*property.name(), property);
      }
      // ...
  }
  ```

- [ ] 4.3: Update expand_property to use PropertyResolver
  ```rust
  RawProperty::Ref(ref_entry) => {
      let prop_ref = BankPropertyRef::try_from(ref_entry.ref_path.as_ref())?;
      let bank_name = prop_ref.name();
      let base = self.bank.get(bank_name).ok_or_else(|| {
          SchemaError::PropertyRefNotFound(ref_entry.ref_path.to_string())
      })?;
      PropertyResolver::resolve_from_bank_ref(base, &ref_entry)
  }
  ```

- [ ] 4.4: Delete old methods
  - Remove apply_ref_overrides
  - Remove apply_spec_overrides
  - Remove type_mismatch helper (moved to PropertyResolver)

### Phase 5: Update Tests ⏱️ 30 min

- [ ] 5.1: Move resolver tests to merger tests
  - Copy test module from old resolver.rs to merger.rs
  - Update imports
  - Rename `mod resolve` assertions

- [ ] 5.2: Update test fixtures
  **File**: `lithos-core/tests/common/mod.rs`
  ```rust
  pub fn schema_with_props(
      name: &str,
      properties: Vec<Property>,
  ) -> Result<Schema, SchemaError> {
      let props_map: HashMap<PropertyName, Property> = properties
          .into_iter()
          .map(|p| (*p.name(), p))
          .collect();

      Ok(Schema::new(
          SchemaId::new(),
          SchemaName::try_new(name)?,
          None,
          vec![],
          props_map,
      ))
  }
  ```

- [ ] 5.3: Remove property ordering tests
  **File**: `lithos-core/src/schema/expander.rs`
  ```rust
  // DELETE:
  #[test]
  fn properties_sorted_by_name() { ... }
  ```

- [ ] 5.4: Update integration tests
  **File**: `lithos-core/tests/schema_resolution.rs`
  - Replace Vec[index] assertions with HashMap.get() checks
  - Update schema construction to use HashMap

- [ ] 5.5: Update Extender tests
  **File**: `lithos-core/src/schema/extender.rs`
  - Update build_nodes to clone HashMap
  - Update any property assertions

### Phase 6: Update Loader ⏱️ 15 min

- [ ] 6.1: Update imports
  **File**: `lithos-core/src/schema/loader.rs`
  ```rust
  use super::merger::Merger;  // Was: resolver::Resolver
  ```

- [ ] 6.2: Update Resolver → Merger calls
  ```rust
  // From:
  let resolved = Resolver::resolve(&tree, &known_parents)?;

  // To:
  let resolved = Merger::resolve(&tree, &known_parents)?;
  ```

- [ ] 6.3: Simplify store_expanded_properties
  ```rust
  fn store_expanded_properties(&self, expanded: &[(SchemaId, RefExpandedSchema)]) {
      for (id, exp_schema) in expanded {
          // exp_schema.properties is already HashMap - use directly!
          if let Some(mut view) = self.ingestor.repository().get_raw_schema_view(*id)? {
              if let Some(current) = view.current_mut() {
                  current.set_expanded_properties(exp_schema.properties.clone());
              }
              self.ingestor.repository().save_raw_schema_view(*id, &view)?;
          }
      }
  }
  ```

### Phase 7: Verification ⏱️ 15 min

- [ ] 7.1: Format code
  ```bash
  cargo fmt --all
  ```

- [ ] 7.2: Run clippy
  ```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

- [ ] 7.3: Run all tests
  ```bash
  cargo nextest run --workspace
  ```

- [ ] 7.4: Verify test count (should be ~791 or more with new PropertyResolver tests)

- [ ] 7.5: Run specific test modules
  ```bash
  cargo nextest run -p lithos-core --lib schema::resolver
  cargo nextest run -p lithos-core --lib schema::merger
  cargo nextest run -p lithos-core --lib schema::expander
  ```

### Phase 8: Commit & Document ⏱️ 15 min

- [ ] 8.1: Review all changes
  ```bash
  git diff --stat
  git diff lithos-core/src/schema/
  ```

- [ ] 8.2: Commit with detailed message
  ```bash
  git add -A
  git commit -m "refactor(schema): switch to HashMap and extract PropertyResolver

  ## What Changed

  **Collection Types**:
  - Schema.properties: Vec → HashMap (O(1) lookups for LSP)
  - PropertyBank.properties: BTreeMap → HashMap (O(1) vs O(log n))
  - RefExpandedSchema.properties: Vec → HashMap (matches storage)
  - SchemaNode.properties: Vec → HashMap (no sorting needed)
  - HashMetadata.properties: BTreeMap → HashMap (order not required)

  **Architecture**:
  - Created PropertyResolver: Property-level conflict resolution
  - Renamed Resolver → Merger: Schema-level property merging
  - Extracted common override logic from Expander into PropertyResolver

  **Simplifications**:
  - Merger.merge_properties: 50+ lines two-pointer → 15 lines HashMap
  - Expander: Removed duplicate override logic
  - Loader: Direct HashMap storage (no Vec→HashMap conversion)

  ## Why

  **Performance**: O(n) → O(1) property lookups (critical for LSP)
  - With 1,000 properties: ~10ms → ~0.01ms per lookup

  **Clarity**: Property resolver vs Schema merger separation

  **Reusability**: PropertyResolver used by both Expander and Merger

  ## Testing

  - Added PropertyResolver test suite (21 new tests)
  - Moved Merger tests from old Resolver
  - Updated integration tests for HashMap
  - All 791+ tests passing"
  ```

- [ ] 8.3: Delete analysis documents
  ```bash
  rm COLLECTION_ANALYSIS.md
  rm COLLECTION_ANALYSIS_V2.md
  rm DETERMINISM_ANALYSIS.md
  rm DETERMINISM_FINAL_ANALYSIS.md
  rm HASHMAP_REFACTORING_PLAN.md
  rm HASHMAP_REFACTORING_PLAN_V2.md
  ```

### Phase 9: Return to Phase 5.2 ⏱️ 5 min

- [ ] 9.1: Read Phase 5.2 specification
  **File**: `loader-ingestor-refactoring-implementation-plan.md` (line 1490)

- [ ] 9.2: Assess simplifications from HashMap refactoring
  - Cached expanded properties are already HashMap
  - No conversion needed between storage and use
  - Can use HashMap directly in partition logic

- [ ] 9.3: Continue with Phase 5.2 implementation
  **Goal**: Use cached expanded properties to skip RefExpander when PropertyBank is fresh

---

## Verification & Rollback

### Success Criteria

- [ ] All 791+ tests passing
- [ ] No clippy warnings
- [ ] Code formatted correctly
- [ ] PropertyResolver has comprehensive test coverage
- [ ] Merger tests migrated from old Resolver
- [ ] Integration tests updated and passing

### Performance Validation (Optional)

```rust
// Benchmark property lookup (optional - can add later)
#[bench]
fn bench_property_lookup_by_name(b: &mut Bencher) {
    let schema = create_schema_with_n_properties(1000);
    let name = PropertyName::try_new("prop_500").unwrap();

    b.iter(|| {
        schema.find_property_by_name(&name)
    });
}
```

### Rollback Plan

If critical issues arise:
1. Revert the single commit
2. All changes are in schema module (isolated)
3. No external API changes (only internal implementation)

### Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| Test failures due to property order | Update assertions to use HashMap.get() instead of Vec[index] |
| Missing PropertyName in HashMap | Ensure property.name() is used as key consistently |
| Clippy warnings on HashMap iteration | Add #[expect(clippy::iter_over_hash_type)] with reason |
| Import errors after rename | Search and replace "resolver::Resolver" → "merger::Merger" |

---

## Files Modified Summary

### New Files
- `lithos-core/src/schema/resolver.rs` (PropertyResolver - property-level)
- `lithos-core/src/schema/merger.rs` (Merger - renamed from old resolver.rs)

### Modified Files
- `lithos-core/src/schema/mod.rs` (module exports)
- `lithos-core/src/schema/aggregate.rs` (Schema.properties → HashMap)
- `lithos-core/src/schema/bank.rs` (PropertyBank.properties → HashMap)
- `lithos-core/src/schema/expander.rs` (use PropertyResolver, build HashMap)
- `lithos-core/src/schema/extender.rs` (SchemaNode.properties → HashMap)
- `lithos-core/src/schema/loader.rs` (use Merger, simplify storage)
- `lithos-core/src/schema/views/metadata.rs` (HashMetadata.properties → HashMap)
- `lithos-core/tests/common/mod.rs` (test helpers for HashMap)
- `lithos-core/tests/schema_resolution.rs` (integration tests)

### Deleted Files
- Analysis documents (after commit)

---

## Estimated Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| 0 | 1h | Create PropertyResolver + tests |
| 1 | 30min | Rename Resolver → Merger |
| 2 | 45min | Update collections to HashMap |
| 3 | 30min | Refactor Merger |
| 4 | 30min | Refactor Expander |
| 5 | 30min | Update tests |
| 6 | 15min | Update Loader |
| 7 | 15min | Verification |
| 8 | 15min | Commit & cleanup |
| 9 | 5min | Return to Phase 5.2 |
| **Total** | **~4-5 hours** | Full refactoring |

---

## Benefits After Completion

1. **Performance**: O(1) property lookups (critical for LSP queries)
2. **Clarity**: PropertyResolver (property-level) vs Merger (schema-level)
3. **Reusability**: PropertyResolver shared by Expander and Merger
4. **Simplicity**: 50+ line merge → 15 line HashMap logic
5. **Maintainability**: HashMap is idiomatic Rust for key-value access
6. **Phase 5.2 Ready**: Cached HashMap properties usable directly

---

## Next Steps After Completion

**IMMEDIATELY return to Phase 5.2** of loader-ingestor refactoring:

**Location**: `loader-ingestor-refactoring-implementation-plan.md` line 1490

**Goal**: Use cached expanded properties to skip RefExpander when PropertyBank is fresh

**Simplified by HashMap refactoring**:
- No conversion needed (HashMap → HashMap)
- Direct usage in partition logic
- Simplified reconstruction of RefExpandedSchema from cache

---

**END OF PLAN**

Ready to proceed with implementation!
