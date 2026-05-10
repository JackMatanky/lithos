# `rkyv::with` Struct: `AsUnixTime`

[Source](https://docs.rs/rkyv/latest/rkyv/with/struct.AsUnixTime.html)

```rust
pub struct AsUnixTime;
```

A wrapper that converts a [`SystemTime`](https://doc.rust-lang.org/nightly/std/time/struct.SystemTime.html "struct std::time::SystemTime") to a [`Duration`](https://doc.rust-lang.org/nightly/core/time/struct.Duration.html "struct core::time::Duration") since [`UNIX_EPOCH`](https://doc.rust-lang.org/nightly/std/time/constant.UNIX_EPOCH.html "constant std::time::UNIX_EPOCH").

If the serialized time occurs before the UNIX epoch, serialization will panic during `resolve`. The resulting archived time will be an [`ArchivedDuration`](https://docs.rs/rkyv/latest/rkyv/time/struct.ArchivedDuration.html "struct rkyv::time::ArchivedDuration") relative to the UNIX epoch.

## Example

```rust
use rkyv::{Archive, with::AsUnixTime};
use std::time::SystemTime;

#[derive(Archive)]
struct Example {
    #[rkyv(with = AsUnixTime)]
    time: SystemTime,
}
```

## Trait Implementations

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/with.rs.html#468-482)

### impl ArchiveWith<SystemTime> for AsUnixTime

Available on **crate feature `std`** only.

The archived type of `Self` with `F`.

The resolver of a `Self` with `F`.

Resolves the archived type using a reference to the field type `F`.

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/with.rs.html#498-510)

### impl<D> DeserializeWith<ArchivedDuration, SystemTime, D> for AsUnixTimewhere D: Fallible +?Sized,

Available on **crate feature `std`** only.

Deserializes the field type `F` using the given deserializer.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/with.rs.html#484-496)

### impl<S> SerializeWith<SystemTime, S> for AsUnixTime

Available on **crate feature `std`** only.

Serializes the field type `F` using the given serializer.

## Auto Trait Implementations

## Blanket Implementations

[Source](https://doc.rust-lang.org/nightly/src/core/any.rs.html#141)

### impl<T> Any for Twhere T: 'static +?Sized,

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#55-62)

### impl<T> ArchivePointee for T

The archived version of the pointer metadata for this type.

Converts some archived metadata to the pointer metadata for itself.

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#212)

### impl<T> Borrow<T> for Twhere T:?Sized,

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#221)

### impl<T> BorrowMut<T> for Twhere T:?Sized,

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#222)

#### fn borrow\_mut(&mut self) -> &mut T

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#786)

### impl<T> From<T> for T

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#789)

#### fn from(t: T) -> T

Returns the argument unchanged.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#768-770)

### impl<T, U> Into<U> for Twhere U: From<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#778)

#### fn into(self) -> U

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `From<T> for U` chooses to do.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#30-36)

### impl<T> LayoutRaw for T

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/with/niching.rs.html#132-145)

### impl<T, N1, N2> Niching<NichedOption<T, N1>> for N2where T: SharedNiching<N1, N2>, N1: Niching<T>, N2: Niching<T>,

Returns whether the given value has been niched. [Read more](https://docs.rs/rkyv/latest/rkyv/niche/niching/trait.Niching.html#tymethod.is_niched)

Writes data to `out` indicating that a `T` is niched.

[Source](https://docs.rs/ptr_meta/0.3.1/x86_64-unknown-linux-gnu/src/ptr_meta/lib.rs.html#141)

### impl<T> Pointee for T

The metadata type for pointers and references to this type.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#828-830)

### impl<T, U> TryFrom<U> for Twhere U: Into<T>,

The type returned in the event of a conversion error.

Performs the conversion.

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#812-814)

### impl<T, U> TryInto<U> for Twhere U: TryFrom<T>,

The type returned in the event of a conversion error.

Performs the conversion.
