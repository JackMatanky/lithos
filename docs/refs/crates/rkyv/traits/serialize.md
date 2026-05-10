# Serialize in rkyv

## Trait Serialize

[Source](https://docs.rs/rkyv/latest/rkyv/trait.Serialize.html)

```rust
pub trait Serialize<S: Fallible + ?Sized>: Archive {
    // Required method
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error>;
}
```

Converts a type to its archived form.

Objects perform any supportive serialization during [`serialize`](https://docs.rs/rkyv/latest/rkyv/trait.Serialize.html#tymethod.serialize "method rkyv::Serialize::serialize"). For types that reference nonlocal (pointed-to) data, this is when that data must be serialized to the output. These types will need to bound `S` to implement [`Writer`](https://docs.rs/rkyv/latest/rkyv/ser/trait.Writer.html "trait rkyv::ser::Writer") and any other required traits (e.g. [`Sharing`](https://docs.rs/rkyv/latest/rkyv/ser/trait.Sharing.html "trait rkyv::ser::Sharing")). They should then serialize their dependencies during `serialize`.

See [`Archive`](https://docs.rs/rkyv/latest/rkyv/trait.Archive.html "trait rkyv::Archive") for examples of implementing `Serialize`.

## Required Methods

Writes the dependencies for the object and returns a resolver that can create the archived type.

## Dyn Compatibility

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementations on Foreign Types

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/btree_set.rs.html#28-44)

### impl<K, S> Serialize<S> for BTreeSet<K>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/collections/hash_set.rs.html#33-50)

### impl<K, S, RS> Serialize<S> for HashSet<K, RS>

Available on **crate feature `std`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/btree_map.rs.html#24-41)

### impl<K, V, S> Serialize<S> for BTreeMap<K, V>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/collections/hash_map.rs.html#28-49)

### impl<K, V, S, RandomState> Serialize<S> for HashMap<K, V, RandomState>

Available on **crate feature `std`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/ffi.rs.html#29-36)

### impl<S: Fallible + Writer +?Sized> Serialize<S> for CString

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#173)

### impl<T0, T1, S> Serialize<S> for (T0, T1)where T0: Serialize<S>, T1: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#174)

### impl<T0, T1, T2, S> Serialize<S> for (T0, T1, T2)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#175)

### impl<T0, T1, T2, T3, S> Serialize<S> for (T0, T1, T2, T3)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#176)

### impl<T0, T1, T2, T3, T4, S> Serialize<S> for (T0, T1, T2, T3, T4)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#177)

### impl<T0, T1, T2, T3, T4, T5, S> Serialize<S> for (T0, T1, T2, T3, T4, T5)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#178)

### impl<T0, T1, T2, T3, T4, T5, T6, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#179)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6, T7)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, T7: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#180-182)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6, T7, T8)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, T7: Serialize<S>, T8: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#183-185)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, T7: Serialize<S>, T8: Serialize<S>, T9: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#186-189)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, T7: Serialize<S>, T8: Serialize<S>, T9: Serialize<S>, T10: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#190-193)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, T7: Serialize<S>, T8: Serialize<S>, T9: Serialize<S>, T10: Serialize<S>, T11: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#194-197)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, S> Serialize<S> for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)where T0: Serialize<S>, T1: Serialize<S>, T2: Serialize<S>, T3: Serialize<S>, T4: Serialize<S>, T5: Serialize<S>, T6: Serialize<S>, T7: Serialize<S>, T8: Serialize<S>, T9: Serialize<S>, T10: Serialize<S>, T11: Serialize<S>, T12: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/boxed.rs.html#26-37)

### impl<T, S> Serialize<S> for Box<T>where T: SerializeUnsized<S> +?Sized, S: Fallible +?Sized,

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/vec_deque.rs.html#27-48)

### impl<T, S> Serialize<S> for VecDeque<T>where T: Serialize<S>, S: Fallible + Allocator + Writer +?Sized,

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/mod.rs.html#34-49)

### impl<T, S> Serialize<S> for Rc<T>where T: SerializeUnsized<S> +?Sized + 'static, S: Fallible + Writer + Sharing +?Sized, S::Error: Source,

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/mod.rs.html#120-135)

### impl<T, S> Serialize<S> for Weak<T>where T: SerializeUnsized<S> +?Sized + 'static, S: Fallible + Writer + Sharing +?Sized, S::Error: Source,

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/atomic.rs.html#31-46)

### impl<T, S> Serialize<S> for Arc<T>where T: SerializeUnsized<S> +?Sized + 'static, S: Fallible + Writer + Sharing +?Sized, S::Error: Source,

Available on **crate feature `alloc` and `target_has_atomic=ptr`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/atomic.rs.html#120-135)

### impl<T, S> Serialize<S> for Weak<T>where T: SerializeUnsized<S> +?Sized + 'static, S: Fallible + Writer + Sharing +?Sized, S::Error: Source,

Available on **crate feature `alloc` and `target_has_atomic=ptr`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#221-239)

### impl<T, S, const N: usize> Serialize<S> for \[T; N\]where T: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/result.rs.html#63-78)

### impl<T, U, S> Serialize<S> for Result<T, U>where T: Serialize<S>, U: Serialize<S>, S: Fallible +?Sized,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/vec.rs.html#24-36)

### impl<T: Serialize<S>, S: Fallible + Allocator + Writer +?Sized> Serialize<S> for Vec<T>

Available on **crate feature `alloc`** only.

## Implementors

[Source](https://docs.rs/rkyv/latest/src/rkyv/collections/util.rs.html#56-73)

### impl<S, BK, BV, K, V> Serialize<S> for EntryAdapter<BK, BV, K, V>where S: Fallible +?Sized, BK: Borrow<K>, BV: Borrow<V>, K: Serialize<S>, V: Serialize<S>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/with.rs.html#171-183)
