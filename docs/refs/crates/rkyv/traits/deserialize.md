# Deserialize in rkyv

## Trait Deserialize

[Source](https://docs.rs/rkyv/latest/rkyv/trait.Deserialize.html)

```rust
pub trait Deserialize<T, D: Fallible + ?Sized> {
    // Required method
    fn deserialize(&self, deserializer: &mut D) -> Result<T, D::Error>;
}
```

Converts a type back from its archived form.

Some types may require specific deserializer capabilities, such as `Rc` and `Arc`. In these cases, the deserializer type `D` should be bound so that it implements traits that provide those capabilities (e.g. [`Pooling`](https://docs.rs/rkyv/latest/rkyv/de/trait.Pooling.html "trait rkyv::de::Pooling")).

This can be derived with [`Deserialize`](https://docs.rs/rkyv/latest/rkyv/derive.Deserialize.html "derive rkyv::Deserialize").

## Required Methods

Deserializes using the given deserializer

## Implementations on Foreign Types

## Implementors

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/btree_set.rs.html#46-70)

### impl<K, D> Deserialize<BTreeSet<K>, D> for ArchivedBTreeSet<K::Archived>where K: Archive + Ord, K::Archived: Deserialize<K, D> + Ord, D: Fallible +?Sized,

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/collections/hash_set.rs.html#52-69)

### impl<K, D, S> Deserialize<HashSet<K, S>, D> for ArchivedHashSet<K::Archived>where K: Archive + Hash + Eq, K::Archived: Deserialize<K, D> + Hash + Eq, D: Fallible +?Sized, S: Default + BuildHasher,

Available on **crate feature `std`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/btree_map.rs.html#43-74)

### impl<K, V, D> Deserialize<BTreeMap<K, V>, D> for ArchivedBTreeMap<K::Archived, V::Archived>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/collections/hash_map.rs.html#51-75)

### impl<K, V, D, S> Deserialize<HashMap<K, V, S>, D> for ArchivedHashMap<K::Archived, V::Archived>

Available on **crate feature `std`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#174)

### impl<T0, T1, T2, D> Deserialize<(T0, T1, T2), D> for ArchivedTuple3<T0::Archived, T1::Archived, T2::Archived>

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#175)

### impl<T0, T1, T2, T3, D> Deserialize<(T0, T1, T2, T3), D> for ArchivedTuple4<T0::Archived, T1::Archived, T2::Archived, T3::Archived>

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#176)

### impl<T0, T1, T2, T3, T4, D> Deserialize<(T0, T1, T2, T3, T4), D> for ArchivedTuple5<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#177)

### impl<T0, T1, T2, T3, T4, T5, D> Deserialize<(T0, T1, T2, T3, T4, T5), D> for ArchivedTuple6<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#178)

### impl<T0, T1, T2, T3, T4, T5, T6, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6), D> for ArchivedTuple7<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#179)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6, T7), D> for ArchivedTuple8<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived, T7::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>, T7::Archived: Deserialize<T7, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#180-182)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6, T7, T8), D> for ArchivedTuple9<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived, T7::Archived, T8::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>, T7::Archived: Deserialize<T7, D>, T8::Archived: Deserialize<T8, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#183-185)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9), D> for ArchivedTuple10<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived, T7::Archived, T8::Archived, T9::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>, T7::Archived: Deserialize<T7, D>, T8::Archived: Deserialize<T8, D>, T9::Archived: Deserialize<T9, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#186-189)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), D> for ArchivedTuple11<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived, T7::Archived, T8::Archived, T9::Archived, T10::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T10: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>, T7::Archived: Deserialize<T7, D>, T8::Archived: Deserialize<T8, D>, T9::Archived: Deserialize<T9, D>, T10::Archived: Deserialize<T10, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#190-193)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11), D> for ArchivedTuple12<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived, T7::Archived, T8::Archived, T9::Archived, T10::Archived, T11::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T10: Archive, T11: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>, T7::Archived: Deserialize<T7, D>, T8::Archived: Deserialize<T8, D>, T9::Archived: Deserialize<T9, D>, T10::Archived: Deserialize<T10, D>, T11::Archived: Deserialize<T11, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#194-197)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, D> Deserialize<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12), D> for ArchivedTuple13<T0::Archived, T1::Archived, T2::Archived, T3::Archived, T4::Archived, T5::Archived, T6::Archived, T7::Archived, T8::Archived, T9::Archived, T10::Archived, T11::Archived, T12::Archived>where D: Fallible +?Sized, T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T10: Archive, T11: Archive, T12: Archive, T0::Archived: Deserialize<T0, D>, T1::Archived: Deserialize<T1, D>, T2::Archived: Deserialize<T2, D>, T3::Archived: Deserialize<T3, D>, T4::Archived: Deserialize<T4, D>, T5::Archived: Deserialize<T5, D>, T6::Archived: Deserialize<T6, D>, T7::Archived: Deserialize<T7, D>, T8::Archived: Deserialize<T8, D>, T9::Archived: Deserialize<T9, D>, T10::Archived: Deserialize<T10, D>, T11::Archived: Deserialize<T11, D>, T12::Archived: Deserialize<T12, D>,

[Source](https://docs.rs/rkyv/latest/src/rkyv/with.rs.html#185-197)
