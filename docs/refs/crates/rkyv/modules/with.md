# `rkyv` Module: `with`

[Source](https://docs.rs/rkyv/latest/rkyv/with/index.html)

Wrapper type support and commonly used wrappers.

Wrappers can be applied with the `#[rkyv(with = ..)]` attribute in the [`Archive`](https://docs.rs/rkyv/latest/rkyv/derive.Archive.html "derive rkyv::Archive") macro.

## Structs

- [`Acquire`](https://docs.rs/rkyv/latest/rkyv/with/struct.Acquire.html)
  - A type indicating acquire atomic loads.

- [`AsBox`](https://docs.rs/rkyv/latest/rkyv/with/struct.AsBox.html)
  - A wrapper that serializes a field into a box.

- [`AsOwned`](https://docs.rs/rkyv/latest/rkyv/with/struct.AsOwned.html)
  - A wrapper that serializes a `Cow` as if it were owned.

- [`AsString`](https://docs.rs/rkyv/latest/rkyv/with/struct.AsString.html)
  - A wrapper that attempts to convert a type to and from UTF-8.

- [`AsUnixTime`](https://docs.rs/rkyv/latest/rkyv/with/struct.AsUnixTime.html)
  - A wrapper that converts a `SystemTime` to a `Duration` since `UNIX_EPOCH`.

- [`AsVec`](https://docs.rs/rkyv/latest/rkyv/with/struct.AsVec.html)
  - A wrapper that serializes associative containers as a `Vec` of key-value pairs.

- [`AtomicLoad`](https://docs.rs/rkyv/latest/rkyv/with/struct.AtomicLoad.html)
  - A wrapper that archives an atomic by loading its value with a particular ordering.

- [`DefaultNiche`](https://docs.rs/rkyv/latest/rkyv/with/struct.DefaultNiche.html)
  - Default `Niching` for various types.

- [`Identity`](https://docs.rs/rkyv/latest/rkyv/with/struct.Identity.html)
  - A no-op wrapper which uses the default impls for the type.

- [`Inline`](https://docs.rs/rkyv/latest/rkyv/with/struct.Inline.html)
  - A wrapper that serializes a reference inline.

- [`InlineAsBox`](https://docs.rs/rkyv/latest/rkyv/with/struct.InlineAsBox.html)
  - A wrapper that serializes a reference as if it were boxed.

- [`Lock`](https://docs.rs/rkyv/latest/rkyv/with/struct.Lock.html)
  - A wrapper that locks a lock and serializes the value immutably.

- [`Map`](https://docs.rs/rkyv/latest/rkyv/with/struct.Map.html)
  - A wrapper that applies another wrapper to the values contained in a type.

- [`MapKV`](https://docs.rs/rkyv/latest/rkyv/with/struct.MapKV.html)
  - A wrapper that applies key and value wrappers to the key-value pairs contained in a type.

- [`MapNiche`](https://docs.rs/rkyv/latest/rkyv/with/struct.MapNiche.html)
  - A wrapper that first applies another wrapper `W` to the value inside an `Option` and then niches the result.

- [`Niche`](https://docs.rs/rkyv/latest/rkyv/with/struct.Niche.html)
  - A wrapper that niches some type combinations.

- [`NicheInto`](https://docs.rs/rkyv/latest/rkyv/with/struct.NicheInto.html)
  - A wrapper that niches based on a generic `Niching`.

- [`Relaxed`](https://docs.rs/rkyv/latest/rkyv/with/struct.Relaxed.html)
  - A type indicating relaxed atomic loads.

- [`SeqCst`](https://docs.rs/rkyv/latest/rkyv/with/struct.SeqCst.html)
  - A type indicating sequentially-consistent atomic loads.

- [`Skip`](https://docs.rs/rkyv/latest/rkyv/with/struct.Skip.html)
  - A wrapper that skips serializing a field.

- [`Unsafe`](https://docs.rs/rkyv/latest/rkyv/with/struct.Unsafe.html)
  - A wrapper that allows serialize-unsafe types to be serialized.

- [`Unshare`](https://docs.rs/rkyv/latest/rkyv/with/struct.Unshare.html)
  - A wrapper that clones the contents of `Arc` and `Rc` pointers.

- [`With`](https://docs.rs/rkyv/latest/rkyv/with/struct.With.html)
  - A transparent wrapper which applies a “with” type.

# Traits

- [`ArchiveWith`](https://docs.rs/rkyv/latest/rkyv/with/trait.ArchiveWith.html)
  - A variant of `Archive` that works with wrappers.

- [`DeserializeWith`](https://docs.rs/rkyv/latest/rkyv/with/trait.DeserializeWith.html)
  - A variant of `Deserialize` for “with” types.

- [`SerializeWith`](https://docs.rs/rkyv/latest/rkyv/with/trait.SerializeWith.html)
  - A variant of `Serialize` for “with” types.
