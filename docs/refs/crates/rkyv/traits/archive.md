# Archive in rkyv

## Trait Archive

[Source](https://docs.rs/rkyv/latest/rkyv/trait.Archive.html)

```rust
pub trait Archive {
    type Archived: Portable;
    type Resolver;

    const COPY_OPTIMIZATION: CopyOptimization<Self> = _;

    // Required method
    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>);
}
```

A type that can be used without deserializing.

`Archive` is one of three basic traits used to work with zero-copy data and controls the layout of the data in its archived zero-copy representation. The [`Serialize`](https://docs.rs/rkyv/latest/rkyv/trait.Serialize.html "trait rkyv::Serialize") trait helps transform types into that representation, and the [`Deserialize`](https://docs.rs/rkyv/latest/rkyv/trait.Deserialize.html "trait rkyv::Deserialize") trait helps transform types back out.

Types that implement `Archive` must have a well-defined archived size. Unsized types can be supported using the [`ArchiveUnsized`](https://docs.rs/rkyv/latest/rkyv/trait.ArchiveUnsized.html "trait rkyv::ArchiveUnsized") trait, along with [`SerializeUnsized`](https://docs.rs/rkyv/latest/rkyv/trait.SerializeUnsized.html "trait rkyv::SerializeUnsized") and [`DeserializeUnsized`](https://docs.rs/rkyv/latest/rkyv/trait.DeserializeUnsized.html "trait rkyv::DeserializeUnsized").

Archiving is done depth-first, writing any data owned by a type before writing the data for the type itself. The type must be able to create the archived type from only its own data and its resolver.

Archived data is always treated as if it is tree-shaped, with the root owning its direct descendents and so on. Data that is not tree-shaped can be supported using special serializer and deserializer bounds (see [`ArchivedRc`](https://docs.rs/rkyv/latest/rkyv/rc/struct.ArchivedRc.html "struct rkyv::rc::ArchivedRc") for example). In a buffer of serialized data, objects are laid out in *reverse order*. This means that the root object is located near the end of the buffer and leaf objects are located near the beginning.

## Examples

Most of the time, `#[derive(Archive)]` will create an acceptable implementation. You can use the `#[rkyv(...)]` attribute to control how the implementation is generated. See the [`Archive`](https://docs.rs/rkyv/latest/rkyv/derive.Archive.html "derive rkyv::Archive") derive macro for more details.

```rust
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
struct Test {
    int: u8,
    string: String,
    option: Option<Vec<i32>>,
}

fn main() {
    let value = Test {
        int: 42,
        string: "hello world".to_string(),
        option: Some(vec![1, 2, 3, 4]),
    };

    // Serializing is as easy as a single function call
    let _bytes = rkyv::to_bytes::<Error>(&value).unwrap();

    // Or you can customize your serialization for better performance or control
    // over resource usage
    use rkyv::{api::high::to_bytes_with_alloc, ser::allocator::Arena};

    let mut arena = Arena::new();
    let bytes =
        to_bytes_with_alloc::<_, Error>(&value, arena.acquire()).unwrap();

    // You can use the safe API for fast zero-copy deserialization
    let archived = rkyv::access::<ArchivedTest, Error>(&bytes[..]).unwrap();
    assert_eq!(archived, &value);

    // Or you can use the unsafe API for maximum performance
    let archived =
        unsafe { rkyv::access_unchecked::<ArchivedTest>(&bytes[..]) };
    assert_eq!(archived, &value);

    // And you can always deserialize back to the original type
    let deserialized = deserialize::<Test, Error>(archived).unwrap();
    assert_eq!(deserialized, value);
}
```

*Note: the safe API requires the `bytecheck` feature.*

Many of the core and standard library types already have `Archive` implementations available, but you may need to implement `Archive` for your own types in some cases the derive macro cannot handle.

In this example, we add our own wrapper that serializes a `&'static str` as if it’s owned. Normally you can lean on the archived version of `String` to do most of the work, or use the [`Inline`](https://docs.rs/rkyv/latest/rkyv/with/struct.Inline.html "struct rkyv::with::Inline") to do exactly this. This example does everything to demonstrate how to implement `Archive` for your own types.

```rust
use core::{slice, str};

use rkyv::{
    access_unchecked,
    rancor::{Error, Fallible},
    ser::Writer,
    to_bytes,
    Archive, ArchiveUnsized, Archived, Portable, RelPtr, Serialize,
    SerializeUnsized, munge::munge, Place,
};

struct OwnedStr {
    inner: &'static str,
}

#[derive(Portable)]
#[repr(transparent)]
struct ArchivedOwnedStr {
    // This will be a relative pointer to our string
    ptr: RelPtr<str>,
}

impl ArchivedOwnedStr {
    // This will help us get the bytes of our type as a str again.
    fn as_str(&self) -> &str {
        unsafe {
            // The as_ptr() function of RelPtr will get a pointer the str
            &*self.ptr.as_ptr()
        }
    }
}

struct OwnedStrResolver {
    // This will be the position that the bytes of our string are stored at.
    // We'll use this to resolve the relative pointer of our
    // ArchivedOwnedStr.
    pos: usize,
}

// The Archive implementation defines the archived version of our type and
// determines how to turn the resolver into the archived form. The Serialize
// implementations determine how to make a resolver from the original value.
impl Archive for OwnedStr {
    type Archived = ArchivedOwnedStr;
    // This is the resolver we can create our Archived version from.
    type Resolver = OwnedStrResolver;

    // The resolve function consumes the resolver and produces the archived
    // value at the given position.
    fn resolve(
        &self,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        munge!(let ArchivedOwnedStr { ptr } = out);
        RelPtr::emplace_unsized(
            resolver.pos,
            self.inner.archived_metadata(),
            ptr,
        );
    }
}

// We restrict our serializer types with Writer because we need its
// capabilities to serialize the inner string. For other types, we might
// need more or less restrictive bounds on the type of S.
impl<S: Fallible + Writer + ?Sized> Serialize<S> for OwnedStr {
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        // This is where we want to write the bytes of our string and return
        // a resolver that knows where those bytes were written.
        // We also need to serialize the metadata for our str.
        Ok(OwnedStrResolver {
            pos: self.inner.serialize_unsized(serializer)?,
        })
    }
}

const STR_VAL: &'static str = "I'm in an OwnedStr!";
let value = OwnedStr { inner: STR_VAL };
// It works!
let buf = to_bytes::<Error>(&value).expect("failed to serialize");
let archived =
    unsafe { access_unchecked::<ArchivedOwnedStr>(buf.as_ref()) };
// Let's make sure our data got written correctly
assert_eq!(archived.as_str(), STR_VAL);
```

## Provided Associated Constants

An optimization flag that allows the bytes of this type to be copied directly to a writer instead of calling `serialize`.

This optimization is disabled by default. To enable this optimization, you must unsafely attest that `Self` is trivially copyable using [`CopyOptimization::enable`](https://docs.rs/rkyv/latest/rkyv/traits/struct.CopyOptimization.html#method.enable "associated function rkyv::traits::CopyOptimization::enable") or [`CopyOptimization::enable_if`](https://docs.rs/rkyv/latest/rkyv/traits/struct.CopyOptimization.html#method.enable_if "associated function rkyv::traits::CopyOptimization::enable_if").

## Required Associated Types

The archived representation of this type.

In this form, the data can be used with zero-copy deserialization.

[Source](https://docs.rs/rkyv/latest/src/rkyv/traits.rs.html#238)

#### type

The resolver for this type. It must contain all the additional information from serializing needed to make the archived type from the normal type.

## Required Methods

Creates the archived version of this value at the given position and writes it to the given output.

The output should be initialized field-by-field rather than by writing a whole struct. Performing a typed copy will mark all of the padding bytes as uninitialized, but they must remain set to the value they currently have. This prevents leaking uninitialized memory to the final archive.

## Dyn Compatibility

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementations on Foreign Types

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/ffi.rs.html#19-27)

### impl Archive for CString

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/string.rs.html#12-20)

### impl Archive for String

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/collections/hash_set.rs.html#15-31)

### impl<K, S> Archive for HashSet<K, S>

Available on **crate feature `std`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/std/collections/hash_map.rs.html#15-26)

### impl<K, V: Archive, S> Archive for HashMap<K, V, S>

Available on **crate feature `std`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/btree_set.rs.html#12-26)

### impl<K: Archive + Ord> Archive for BTreeSet<K>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/btree_map.rs.html#12-22)

### impl<K: Archive + Ord, V: Archive> Archive for BTreeMap<K, V>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#173)

### impl<T0, T1> Archive for (T0, T1)where T0: Archive, T1: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#174)

### impl<T0, T1, T2> Archive for (T0, T1, T2)where T0: Archive, T1: Archive, T2: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#175)

### impl<T0, T1, T2, T3> Archive for (T0, T1, T2, T3)where T0: Archive, T1: Archive, T2: Archive, T3: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#176)

### impl<T0, T1, T2, T3, T4> Archive for (T0, T1, T2, T3, T4)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#177)

### impl<T0, T1, T2, T3, T4, T5> Archive for (T0, T1, T2, T3, T4, T5)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#178)

### impl<T0, T1, T2, T3, T4, T5, T6> Archive for (T0, T1, T2, T3, T4, T5, T6)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#179)

### impl<T0, T1, T2, T3, T4, T5, T6, T7> Archive for (T0, T1, T2, T3, T4, T5, T6, T7)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#180-182)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8> Archive for (T0, T1, T2, T3, T4, T5, T6, T7, T8)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#183-185)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9> Archive for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#186-189)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Archive for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T10: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#190-193)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> Archive for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T10: Archive, T11: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/core/mod.rs.html#194-197)

### impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> Archive for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)where T0: Archive, T1: Archive, T2: Archive, T3: Archive, T4: Archive, T5: Archive, T6: Archive, T7: Archive, T8: Archive, T9: Archive, T10: Archive, T11: Archive, T12: Archive,

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/collections/vec_deque.rs.html#18-25)

### impl<T: Archive> Archive for VecDeque<T>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/vec.rs.html#15-22)

### impl<T: Archive> Archive for Vec<T>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/boxed.rs.html#17-24)

### impl<T: ArchiveUnsized +?Sized> Archive for Box<T>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/mod.rs.html#25-32)

### impl<T: ArchiveUnsized +?Sized> Archive for Rc<T>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/mod.rs.html#107-118)

### impl<T: ArchiveUnsized +?Sized> Archive for Weak<T>

Available on **crate feature `alloc`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/atomic.rs.html#22-29)

### impl<T: ArchiveUnsized +?Sized> Archive for Arc<T>

Available on **crate feature `alloc` and `target_has_atomic=ptr`** only.

[Source](https://docs.rs/rkyv/latest/src/rkyv/impls/alloc/rc/atomic.rs.html#107-118)

### impl<T: ArchiveUnsized +?Sized> Archive for Weak<T>

Available on **crate feature `alloc` and `target_has_atomic=ptr`** only.
