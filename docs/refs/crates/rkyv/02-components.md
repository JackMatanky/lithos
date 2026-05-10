# rkyv Components Index

This document provides a comprehensive index of the core components of the `rkyv` crate based on its documentation.

## 1. Traits

The core traits define the fundamental behaviors for archiving and transforming types into their zero-copy representations.

* **[`Archive`](https://docs.rs/rkyv/latest/rkyv/traits/trait.Archive.html)**
  * **Purpose:** Defines the interface for types that can be archived into a zero-copy representation.
  * **Features:** Controls the layout of data in its archived form and specifies the associated `Archived` type and `Resolver` type needed for transformation. Uses the `resolve` method to finalize data placement.
* **[`Serialize`](https://docs.rs/rkyv/latest/rkyv/traits/trait.Serialize.html)**
  * **Purpose:** Converts a given Rust type into its archived form.
  * **Features:** Generates the `Resolver` for the type, converting complex or heap-allocated structures into a format suitable for the final archived representation.
* **[`Deserialize`](https://docs.rs/rkyv/latest/rkyv/traits/trait.Deserialize.html)**
  * **Purpose:** Converts an archived type back into its original Rust type.
  * **Features:** Essential for mutating data or utilizing standard library functions that require native, un-archived Rust types.
* **[`Portable`](https://docs.rs/rkyv/latest/rkyv/traits/trait.Portable.html)**
  * **Purpose:** Guarantees a stable, well-defined memory layout that is identical across architectures.
  * **Features:** Safely enables transmission of serialized data between machines with varying endianness and ensures safe cross-platform memory mapping.

## 2. Collections

`rkyv` provides archived equivalents of standard Rust collections to allow querying and access without any allocation overhead.

* **[`ArchivedVec`](https://docs.rs/rkyv/latest/rkyv/collections/struct.ArchivedVec.html)**
  * **Purpose:** The archived representation of a `Vec<T>`.
  * **Features:** Allows zero-copy iteration and `O(1)` indexing over elements. Elements are stored contiguously in memory.
* **[`ArchivedString`](https://docs.rs/rkyv/latest/rkyv/string/struct.ArchivedString.html)**
  * **Purpose:** The archived representation of a `String`.
  * **Features:** Highly efficient, supporting *inline short strings* to avoid out-of-line pointer chasing for small strings.
* **[`ArchivedHashMap`](https://docs.rs/rkyv/latest/rkyv/collections/hash_map/struct.ArchivedHashMap.html)**
  * **Purpose:** The archived representation of a `HashMap<K, V>`.
  * **Features:** Enables `O(1)` average-time-complexity lookups for keys directly from archived bytes.
* **[`ArchivedBTreeMap`](https://docs.rs/rkyv/latest/rkyv/collections/btree_map/struct.ArchivedBTreeMap.html)**
  * **Purpose:** The archived representation of a `BTreeMap<K, V>`.
  * **Features:** Provides zero-copy iteration in sorted order. Safely implements `Portable`, `Send`, and `Sync`.

## 3. Wrappers

Wrappers modify how specific fields of a struct or enum are archived (usually via `#[rkyv(with = ...)]`).

* **[`Inline`](https://docs.rs/rkyv/latest/rkyv/with/struct.Inline.html)**
  * **Purpose:** Directs the serializer to inline the data rather than storing it out-of-line behind a relative pointer.
  * **Features:** Reduces pointer indirection overhead by archiving directly into the struct's layout.
* **[`Skip`](https://docs.rs/rkyv/latest/rkyv/with/struct.Skip.html)**
  * **Purpose:** Instructs the serializer to skip archiving a specific field.
  * **Features:** The resulting archived type for that field becomes `()`.
* **[`Raw`](https://docs.rs/rkyv/latest/rkyv/with/struct.Raw.html)**
  * **Purpose:** Handles unadulterated byte data or raw pointers.
  * **Features:** Utilized for archiving raw unmanaged data without recursive archiving logic.

## 4. Fundamental Types

These foundational types make relative addressing and type resolution possible behind the scenes.

* **[`RelPtr`](https://docs.rs/rkyv/latest/rkyv/rel_ptr/struct.RelPtr.html)**
  * **Purpose:** A relative pointer (aliased as `RelPtr<T, ArchivedIsize>`).
  * **Features:** Stores the offset from its own position to the target data instead of an absolute memory address. This core mechanic allows `rkyv` to be safely memory-mapped anywhere.
* **[`Archived`](https://docs.rs/rkyv/latest/rkyv/type.Archived.html)**
  * **Purpose:** A convenience type alias for `<T as Archive>::Archived`.
  * **Features:** Reduces verbosity when referring to the archived version of a type `T` (e.g., `Archived<T>`).
* **[`Place`](https://docs.rs/rkyv/latest/rkyv/struct.Place.html)**
  * **Purpose:** Represents a location in memory where the archived type will be written.
  * **Features:** Used strictly during the `resolve` phase to concretely construct relative pointers and data layouts.
