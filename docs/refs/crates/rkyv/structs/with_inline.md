# `rkyv::with` Struct: `Inline`

[Source](https://docs.rs/rkyv/latest/rkyv/with/struct.Inline.html)

```rust
pub struct Inline;
```

A wrapper that serializes a reference inline.

References serialized with `Inline` cannot be deserialized because the struct cannot own the deserialized value.

## Example

```rust
use rkyv::{with::Inline, Archive};

#[derive(Archive)]
struct Example<'a> {
    #[rkyv(with = Inline)]
    a: &'a i32,
}
```

## Trait Implementations

The archived type of `Self` with `F`.

The resolver of a `Self` with `F`.

Resolves the archived type using a reference to the field type `F`.

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

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
