# rkyv

[rkyv](http://github.com/rkyv/rkyv) (_archive_) is a zero-copy deserialization framework for Rust.

This book covers the motivation, architecture, and major features of rkyv. It is the best way to learn and understand rkyv, but won't go as in-depth on specifics as the documentation will. Don't be afraid to consult these other resources as you need while you read through.

## 1. Resources

### 1.1. Learning Materials

- The [rkyv discord](https://discord.gg/65F6MdnbQh) is a great place to get help with specific issues and meet other people using rkyv
- The [rkyv github](https://github.com/rkyv/rkyv) hosts the source and tracks project issues and milestones.
- There are examples of usage in [the repository](https://github.com/rkyv/rkyv/tree/main/rkyv/examples).

### 1.2. Documentation

- [rkyv](https://docs.rs/rkyv), the core library
- [rkyv_dyn](https://docs.rs/rkyv_dyn), which adds trait object support to rkyv

### 1.3. Benchmarks

- The [rust serialization benchmark](https://github.com/djkoloski/rust_serialization_benchmark) is a shootout style benchmark comparing many rust serialization solutions. It includes special benchmarks for zero-copy serialization solutions like rkyv.

### 1.4. Sister Crates

- [rend](https://github.com/rkyv/rend), which rkyv uses for endian-agnostic features
- [bytecheck](https://github.com/rkyv/bytecheck), which rkyv uses for validation
- [rancor](https://github.com/rkyv/rancor), which rkyv uses for error handling
- [ptr_meta](https://github.com/rkyv/ptr_meta), which rkyv uses for pointer manipulation

## 2. Motivation

First and foremost, the motivation behind rkyv is improved performance. The way that it achieves that goal can also lead to gains in memory use, correctness, and security along the way.

> Familiarity with other serialization frameworks and how traditional serialization works will help, but isn't necessary to understand how rkyv works.

Most serialization frameworks like [serde](https://serde.rs/) define an internal data model that consists of basic types such as primitives, strings, and byte arrays. This splits the work of serializing a type into two stages: the frontend and the backend. The frontend takes some type and breaks it down into the serializable types of the data model. The backend then takes the data model types and writes them using some data format such as JSON, Bincode, TOML, etc. This allows a clean separation between the serialization of a type and the data format it is written to.

> Serde describes [its data model](https://serde.rs/data-model.html) in the serde book. Everything serialized with serde eventually boils down to some combination of those types!

A major downside of traditional serialization is that it takes a considerable amount of time to read, parse, and reconstruct types from their serialized values.

> In JSON for example, strings are encoded by surrounding the contents with double quotes and escaping invalid characters inside of them:
>
> ```js
> { "line": "\"All's well that ends well\"" }
>           ^^                          ^ ^
> ```
>
> numbers are turned into characters:
>
> ```js
> { "pi": 3.1415926 }
>         ^^^^^^^^^
> ```
>
> and even field names, which could be _implicit_ in most cases, are turned into strings:
>
> ```js
> { "message_size": 334 }
>   ^^^^^^^^^^^^^^^
> ```
>
> All those characters are not only taking up space, they're also taking up time. Every time we read and parse JSON, we're picking through those characters in order to figure out what the values are and reproduce them in memory. An `f32` is only four bytes of memory, but it's encoded using nine bytes and we still have to turn those nine characters into the right `f32`!

This deserialization time adds up quickly, and in data-heavy applications such as games and media editing it can come to dominate load times. rkyv provides a solution through a serialization technique called _zero-copy deserialization_.

## 3. Zero-copy deserialization

Zero-copy deserialization is a technique that reduces the time and memory required to access and use data by _directly referencing bytes in the serialized form_.

> This takes advantage of how we have to have some data loaded in memory in order to deserialize it. If we had some JSON:
>
> ```js
> { "quote": "I don't know, I didn't listen." }
>             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
> ```
>
> Instead of copying those characters into a `String`, we could just _borrow_ it from the JSON buffer as a `&str`. The lifetime of that `&str` would depend on our buffer and we wouldn't be allowed to drop it until we had dropped the string we were using.

### 3.1. Partial zero-copy

Serde and others have support for partial zero-copy deserialization, where bits and pieces of the deserialized data are borrowed from the serialized form. Strings, for example, can borrow their bytes directly from the serialized form in encodings like bincode that don't perform any character escaping. However, a string object must still be created to hold the deserialized length and point to the borrowed characters.

> A good way to think about this is that even though we're borrowing lots of data from the buffer, we still have to parse the _structure_ out:
>
> ```rust
> struct Example<'a> {
>   quote: &'a str,
>   a: &'a [u8; 12],
>   b: u64,
>   c: char,
> }
> ```
>
> So a buffer might break down like this:
>
> ```js
> I don't know, I didn't listen.AAAAAAAAAAAABBBBBBBBCCCC
> ^-----------------------------^-----------^-------^---
>  quote: str                    a: [u8; 12] b: u64  c: char
> ```
>
> We do a lot less work, but we still have to parse, create, and return an `Example<'a>`:
>
> ```rust
> Example {
>   quote: str::from_utf8(&buffer[0..30]).unwrap(),
>   a: &buffer[30..42],
>   b: u64::from_le_bytes(&buffer[42..50]),
>   c: char::from_u32(u32::from_le_bytes(&buffer[50..54]))).unwrap(),
> }
> ```
>
> And we can't borrow types like `u64` or `char` that have alignment requirements since our buffer might not be properly aligned. We have to immediately parse and store those! Even though we borrowed 42 of the buffer's bytes, we missed out on the last 12 and still had to parse through the buffer to find out where everything is.

Partial zero-copy deserialization can considerably improve memory usage and often speed up some deserialization, but with some work we can go further.

### 3.2. Total zero-copy

rkyv implements total zero-copy deserialization, which guarantees that no data is copied during deserialization and no work is done to deserialize data. It achieves this by structuring its encoded representation so that it is the same as the in-memory representation of the source type.

> This is more like if our buffer _was_ an Example:
>
> ```rust
> struct Example {
>   quote: String,
>   a: [u8; 12],
>   b: u64,
>   c: char,
> }
> ```
>
> And our buffer looked like this:
>
> ```js
> I don't know, I didn't listen.__QOFFQLENAAAAAAAAAAAABBBBBBBBCCCC
> ^-----------------------------  ^---^---^-----------^-------^---
>  quote bytes                    pointer  a           b       c
>                                 and len
>                                 ^-------------------------------
>                                  Example
> ```
>
> In this case, the bytes are padded to the correct alignment and the fields of `Example` are laid out exactly the same as they would be in memory. Our deserialization code can be much simpler:
>
> ```rust
> unsafe { &*buffer.as_ptr().add(32).cast() }
> ```
>
> This operation is almost zero work, and more importantly it doesn't _scale_ with our data. No matter how much or how little data we have, it's always just a pointer offset and a cast to access our data.

This opens up blazingly-fast data loading and enables data access orders of magnitude more quickly than traditional serialization.

## 4. Architecture

The core of rkyv is built around relative pointers and three core traits: `Archive`, `Serialize`, and `Deserialize`. Each of these traits has a corresponding variant that supports unsized types: `ArchiveUnsized`, `SerializeUnsized`, and `DeserializeUnsized`.

> A good way to think about it is that sized types are the _foundation_ that unsized types are built on. That's not a fluke either, rkyv is built precisely so that you can build more complex abstractions out of lower-level machinery in a safe and composable way. It's not much different from what you normally do while programming!

The system is built to be flexible and can be extended beyond the provided types. For example, the `rkyv_dyn` crate adds support for trait objects by introducing new traits and defining how they build up to allow trait objects to be serialized and deserialized.

### 4.1. Relative pointers

Relative pointers are the bread and butter of total zero-copy deserialization, completely replacing the use of normal pointers. But why can't we use normal pointers?

Consider some zero-copy data on disk. Before we can use it, we need to load it into memory. But we can't control _where_ in memory it gets loaded. Every time we load it, it could be located at a different address, and therefore the objects inside of it will be located at a different address.

> One of the major reasons for this is actually _security_. Every time you run your program, it may run in a completely different random location in memory. This is called [address space layout randomization](https://en.wikipedia.org/wiki/Address_space_layout_randomization) and it helps prevent exploitation of memory corruption vulnerabilities.
>
> At most, we can only control the _alignment_ of our zero-copy data, so we need to work within those constraints.

This means that we can't store any pointers to that data, inside of it or outside of it. As soon as we reload the data, it might not be at the same address. That would leave our pointers dangling, and would almost definitely result in memory access violations. Some other libraries like [abomonation](https://github.com/TimelyDataflow/abomonation) store some extra data and perform a fast fixup step that takes the place of deserialization, but we can do better.

> In order to perform that fixup step, abomonation requires that the buffer has a _mutable backing_. This is okay for many use cases, but there are also cases where we won't be able to mutate our buffer. One example is if we used [memory-mapped files](https://en.wikipedia.org/wiki/Memory-mapped_file).

While normal pointers hold an absolute address in memory, relative pointers hold an offset to an address. This changes how the pointer behaves under moves:

| Pointer  | Self is moved                    | Self and target are moved                       |
| -------- | -------------------------------- | ----------------------------------------------- |
| Absolute | ✅ Target is still at address    | ❌ Target no longer at address                  |
| Relative | ❌ Relative distance has changed | ✅ Self and target same relative distance apart |

This is exactly the property we need to build data structures with total zero-copy deserialization. By using relative pointers, we can load data at any position in memory and still have valid pointers inside of it. Relative pointers don't require write access to memory either, so we can memory map entire files and instantly have access to their data in a structured manner.

rkyv's implementation of relative pointers is the `RelPtr` type.

### 4.2. Archive

Types that implement `Archive` have an alternate representation that supports zero-copy deserialization. The construction of archived types happens in two steps:

1. Any dependencies of the type are serialized. For strings this would be the characters of the string, for boxes it would be the boxed value, and for vectors it would be any contained elements. Any bookkeeping from this step is bundled into a `Resolver` type and held onto for later. This is the _serialize_ step.
2. The resolver and original value are used to construct the archived value in the output buffer. For strings the resolver would be the position of the characters, for boxes it would be the position of the boxed value, and for vectors it would be the position of the archived elements. With the original values and resolvers combined, the archived version can be constructed. This is the _resolve_ step.

#### 4.2.1. Resolvers

A good example of why resolvers are necessary is when archiving a tuple. Say we have two strings:

```rust
#![allow(unused)]
fn main() {
let value = ("hello".to_string(), "world".to_string());
}
```

The archived tuple needs to have both of the strings right next to each other:

```js
0x0000      AA AA AA AA BB BB BB BB
0x0008      CC CC CC CC DD DD DD DD
```

A and B might be the length and pointer for the first string of the tuple, and C and D might be the length and pointer for the second string.

When archiving, we might be tempted to serialize and resolve the first string, then serialize and resolve the second one. But this might place the second string's bytes ("world") between the two! Instead, we need to write out the bytes for both strings, and then finish archiving both of them. The tuple doesn't know what information the strings need to finish archiving themselves, so they have to provide it to the tuple through their Resolver.

This way, the tuple can:

1. Archive the first string (save the resolver)
2. Archive the second string (save the resolver)
3. Resolve the first string with its resolver
4. Resolve the second string with its resolver

And we're guaranteed that the two strings are placed right next to each other like we need.

### 4.3. Serialize

Types implement `Serialize` separately from `Archive`. `Serialize` creates a resolver for some object, then `Archive` turns the value and that resolver into an archived type. Having a separate `Serialize` trait is necessary because although a type may have only one archived representation, it may support many different types of _serializers_ which fulfill its requirements.

> The `Serialize` trait is parameterized over the _serializer_. The serializer is just a mutable object that helps the type serialize itself. The most basic types like `u32` or `char` don't _bound_ their serializer type because they can serialize themselves with any kind of serializer. More complex types like `Box` and `String` require a serializer that implements `Writer`, and even more complex types like `Rc` and `Vec` require a serializer that additionally implements `Sharing` or `Allocator`.

Unlike `Serialize`, `Archive` doesn't parameterize over the serializer used to make it. It shouldn't matter what serializer a resolver was made with, only that it's made correctly.

#### 4.3.1. Serializer

rkyv provides default serializers which can serialize all standard library types, as well as components which can be combined into custom-built serializers. By combining rkyv's provided components, serializers can be customized for high-performance, no-std, and custom allocation.

When using the high-level API, a `HighSerializer` provides a good balance of flexibility and performance by default. When using the low-level API, a `LowSerializer` does the same without any allocations. You can make custom serializers using the `Serializer` combinator, or by writing your own from scratch.

rkyv comes with a few primary serializer traits built-in:

##### 4.3.1.1. Positional

This core serializer trait provides positional information during serialization. Because types need to know the relative distance between objects, the `Positional` trait provides the current position of the "write head" of the serializer. Resolvers will often store the _position_ of some serialized data so that a relative pointer can be calculated to it during `resolve`.

##### 4.3.1.2. Writer

`Writer` accepts byte slices and writes them to some output. It is similar to the standard library's `Write` trait, but rkyv's `Writer` trait works in no-std contexts. In rkyv, writers are always _write-forward_ - they never backtrack and rewrite data later. This makes it possible for writers to eagerly sink bytes to disk or the network without having to first buffer the entire message.

Several kinds of `Writer` s are supported by default:

- `Vec<u8>`
- `AlignedVec`, which is a highly-aligned vector of bytes. This is the writer rkyv uses by default in most cases.
- `Buffer`, which supports no-std use cases (for example, writing into fixed-size stack memory).
- Types which implement `std::io::Write` can be adapted into a `Writer` by wrapping them in the `IoWriter` type.

##### 4.3.1.3. Allocator

Many types require temporarily-allocated space during serialization. This space is used temporarily, and then returned to the serializer before serialization finishes. For example, `Vec` might request a dynamically-sized allocation to store the resolvers for its elements until it finishes serializing all of them. Allocating memory from the serializer allows the same bytes to be efficiently reused many times, which reduces the number of slow memory allocations performed during serialization.

##### 4.3.2.4. Sharing

rkyv serializes shared pointers like `Rc` and `Arc` and can control whether they are de-duplicated. The `Sharing` trait provides some mutable state on the serializer which keeps track of which shared pointers have been serialized so far, and can instruct repeated shared pointers to point to a previously-serialized instance. This also allows rkyv to preserve shared pointers during zero-copy access and deserialization.

### 4.4. Deserialize

Similarly to `Serialize`, `Deserialize` parameterizes over a deserializer, and converts a type from its archived form back to its original one. Unlike serialization, deserialization occurs in a single step and doesn't have an equivalent of a resolver.

> `Deserialize` also parameterizes over the type that is being deserialized into. This allows the same archived type to deserialize into multiple different unarchived types depending on what's being asked for. This helps enable lots of very powerful abstractions, but might require you to use a turbofish or annotate types when deserializing.

This provides a more or less traditional deserialization with the added benefit of being sped up by having very compiler-friendly representations. It also incurs both the memory and performance penalties of traditional deserialization, so make sure that it's what you need before you use it. Deserialization is not required to access archived data as long as you can do so through the archived versions.

> Even the highest-performance serialization frameworks will hit a deserialization speed limit because of the amount of memory allocation that needs to be performed.

A good use for `Deserialize` is deserializing small portions of archives. You can easily traverse the archived data to locate some subobject, then deserialize just that piece instead of the archive as a whole. This granular approach provides the benefits of both zero-copy deserialization as well as traditional deserialization.

#### 4.4.1. Pooling

Deserializers, like serializers, provide capabilities to objects during deserialization. Most types don't need to bound their deserializers, but some like `Rc` require special traits in order to deserialize properly.

The `Pooling` trait controls how pointers which were serialized shared are deserialized. Much like `Sharing`, `Pooling` holds some mutable state on the deserializer to allow shared pointers to the same data to coordinate with each other. Using the `Pool` implementation pools these deserialized shared pointers together, whereas `Unpool` clones them for each instance of the shared pointer.

## 5. Format

Types which derive `Archive` generate an archived version of the type where:

- Member types are replaced with their archived counterparts
- Structs are `#[repr(C)]`.
- Enums have `#[repr(N)]` where N is `u8`, `u16`, `u32`, `u64`, or `u128`, choosing the smallest possible type that can represent all of the variants.
- All primitives are replaced with versions which have stable, well-defined layouts and byte orders.

For example, a struct like:

```rust
#![allow(unused)]
fn main() {
struct Example {
    a: u32,
    b: String,
    c: Box<(u32, String)>,
}
}
```

Would have the archived counterpart:

```rust
#![allow(unused)]
fn main() {
#[repr(C)]
struct ArchivedExample {
    a: u32_le,
    b: ArchivedString,
    c: ArchivedBox<ArchivedTuple2<u32_le, ArchivedString>>,
}
}
```

With the `little_endian` feature enabled.

rkyv provides `Archive` implementations for common standard library types by default. In general, they follow the same format as derived implementations but may differ in some cases. For example, `ArchivedString` performs a small string optimization which helps reduce memory use.

### Format control

rkyv provides sets of feature flags which control the basic properties of archived primitives:

- Endianness: `little_endian` / `big_endian` control the endianness of the underlying data
- Alignment: `aligned` / `unaligned` control whether primitive types have alignment greater than 1.
- Pointer width: `pointer_width_16` / `pointer_width_32` / `pointer_width_64` control the size of relative pointer offsets. This allows trading off space for a larger maximum buffer size.

When left unspecified, rkyv chooses these defaults for format control:

- Little-endian
- Aligned
- 32-bit relative pointers

### Object order

rkyv lays out subobjects in depth-first order from the leaves to the root. This means that the root object is stored at the end of the buffer, not the beginning. For example, this tree:

```js
a
 / \
b   c
   / \
  d   e
```

would be laid out like this in the buffer:

```js
b d e c a
```

from this serialization order:

```js
a -> b
a -> c -> d
a -> c -> e
a -> c
a
```

This deterministic layout means that you don't need to store the position of the root object in most cases. As long as your buffer ends right at the end of your root object, you can use `access` with your buffer.

### 5.3. Alignment

The _alignment_ of a type restricts where it can be located in memory to optimize hardware loads and stores. Because rkyv creates references to values located in your serialized bytes, it has to ensure that the references it creates are properly _aligned_ for the type.

> In order to perform arithmetic and logical operations on data, modern CPUs need to _load_ that data from memory into its registers. However, there's usually a hardware limitation on how the CPU can access that data: it can only access data starting at _word boundaries_. These words are the natural size for the CPU to work with; the word size is 4 bytes for 32-bit machines and 8 bytes for 64-bit machines. Imagine we had some data laid out like this:
>
> ```js
> 0   4   8   C
> AAAABBBBCCCCDDDD
> ```
>
> On a 32-bit CPU, accesses could occur at any address that's a multiple of 4 bytes. For example, one could access `A` by loading 4 bytes from address 0, `B` by loading 4 bytes from address 4, and so on. This works great because our data is _aligned_ to word boundaries. _Unaligned_ data can throw a wrench in that:
>
> ```js
> 0   4   8   C
> ..AAAABBBBCCCC
> ```
>
> Now if we want to load `A` into memory, we have to:
>
> 1. Load 4 bytes from address 0
> 2. Throw away the first two bytes
> 3. Load 4 bytes from address 4
> 4. Throw away the last two bytes
> 5. Combine our four bytes together
>
> That forces us to do twice as many loads _and_ perform some correction logic. That can have a real impact on our performance across the board, so we require all of our data to be properly aligned.

rkyv provides two main utilities for aligning byte buffers:

- `AlignedVec`, a higher-aligned drop-in replacement for `Vec<u8>`
- `Align`, a wrapper type which aligns its field to a 16-byte boundary

For most use cases, 16-aligned memory should be sufficient.

#### 5.3.1. In practice

rkyv's unchecked APIs have very basic alignment checks which always run in debug builds. These may not catch every case, but using [validation](#validation) will always make sure that your data is properly aligned.

#### 5.3.2. Common pitfalls

In some cases, your archived data may be prefixed by some extra data like the length of the buffer. If this extra data misaligns the following data, then the buffer will have to have the prefixing data removed before accessing it.

In other cases, your archived data may not be tight to the end of the buffer. Functions like `access` rely on the end of the buffer being tight to the end of the data, and may miscalculate the position of the archived data if it is not.

## 6. Derive macro features

rkyv's derive macro supports a number of attributes and configurable options. All of rkyv's macro attributes are documented on the `Archive` proc-macro. Some of the most important ones to know are:

### `omit_bounds`

rkyv's derive macro performs a "perfect derive" by default. This means that when it generates trait impls, it adds where clauses requiring each field type to also implement that trait. This can cause trouble in two primary situations:

1. Recursive type definitions (using e.g. `Box`) cause an overflow and never finish evaluating
2. Private types may be exposed by these derive bounds.

Both of these situations can be fixed by adding `#[rkyv(omit_bounds)]` on the field. This prevents rkyv from adding the "perfect derive" bounds for that field.

When you do omit the bounds for a particular field, it can lead to insufficient bounds being added to the generated impl. To add custom bounds back, you can use:

- `#[rkyv(archive_bounds(..))]` to add predicates to all generated impls
- `#[rkyv(serialize_bounds(..))]` to add predicates to just the `Serialize` impl
- `#[rkyv(deserialize_bounds(..))]` to add predicates to just the `Deserialize` impl

See `rkyv/examples/json_like_schema.rs` for a fully-commented example of using `omit_bounds`.

### `with =..`

This customizes the serialization of a field by applying a [wrapper type](#wrapper-types).

### `remote =..`

This performs a [remote derive](#remote-derive) for supporting external types.

### `attr(..)` and `derive(..)`

`#[rkyv(attr(..))]` is a general-purpose attribute which allows you to pass attributes down to the generated archived type. This can be especially useful in combination with `#[rkyv(derive(..))]`, which may be used on types and is sugar for `#[rkyv(attr(derive(..)))]`.

### 6.1. Wrapper types

Wrapper types customize the way that fields of types are archived. In some cases, wrapper types merely change the default behavior to a preferred alternative. In other cases, wrapper types allow serializing types which do not have support for rkyv by default.

Annotating a field with `#[rkyv(with = ..)]` will _wrap_ that field with the given types when the struct is serialized or deserialized. There's no performance penalty to wrapping types, but doing more or less work during serialization and deserialization can affect performance. This excerpt is from the documentation for `ArchiveWith`:

```rust
#[derive(Archive, Deserialize, Serialize)]
struct Example {
    #[rkyv(with = Incremented)]
    a: i32,
    // Another i32 field, but not incremented this time
    b: i32,
}
```

The `Incremented` wrapper is wrapping `a`, and the definition causes that field to be incremented in its archived form.

### 6.2. Remote derive

Like serde, rkyv also supports _remote derive_. This allows you to easily generate wrapper types to serialize types from other crates which don't provide rkyv support. Remote derive uses a local definition of the type to serialize, and generates a wrapper type you can use to serialize that type.

Remote derive supports getters, wrapper types, and deserialization back to the original type by providing a `From` impl. This example is from `rkyv/examples/remote_types.rs`:

```rust
#![allow(unused)]
fn main() {
// Let's create a local type that will serve as \`with\`-wrapper for \`Foo\`.
// Fields must have the same name and type but it's not required to define all
// fields.
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = remote::Foo)] // <-
#[rkyv(archived = ArchivedFoo)]
// ^ not necessary but we might as well replace the default name
// \`ArchivedFooDef\` with \`ArchivedFoo\`.
struct FooDef {
    // The field's type implements \`Archive\` and we don't want to apply any
    // conversion for the archived type so we don't need to specify
    // \`#[rkyv(with = ..)]\`.
    ch: char,
    // The field is private in the remote type so we need to specify a getter
    // to access it. Also, its type doesn't implement \`Archive\` so we need
    // to specify a \`with\`-wrapper too.
    #[rkyv(getter = remote::Foo::bar, with = BarDef)]
    bar: remote::Bar<i32>,
    // The remote \`bytes\` field is public but we can still customize our local
    // field when using a getter.
    #[rkyv(getter = get_first_byte)]
    first_byte: u8,
}

fn get_first_byte(foo: &remote::Foo) -> u8 {
    foo.bytes[0]
}

// Deriving \`Deserialize\` with \`remote = ..\` requires a \`From\` implementation.
impl From<FooDef> for remote::Foo {
    fn from(value: FooDef) -> Self {
        remote::Foo::new(value.ch, [value.first_byte, 2, 3, 4], 567, value.bar)
    }
}
}
```

## 7. Shared Pointers

The implementation details of shared pointers may be of interest to those using them. The rules surrounding how and when shared and weak pointers are serialized and pooled may affect how you choose to use them.

### 7.1. Serialization

Shared pointers (`Rc` and `Arc`) are serialized whenever they're encountered for the first time, and the data address is reused when subsequent shared pointers point to the same data. This means that you can expect shared pointers to always point to the same value when archived, even if they are unsized to different types.

Weak pointers (`rc::Weak` and `sync::Weak`) have serialization attempted as soon as they're encountered. The serialization process upgrades them, and if it succeeds it serializes them like shared pointers. Otherwise, it serializes them like `None`.

### 7.2. Deserialization

Similarly, shared pointers are deserialized on the first encounter and reused afterward. Weak pointers do a similar upgrade attempt when they're encountered for the first time.

### 7.3. Serializers and Deserializers

The serializers for shared pointers hold the location of the serialized data. This means it's safe to serialize shared pointers to an archive across multiple `serialize` calls as long as you use the same serializer for each one. Using a new serializer will still do the right thing, but may end up duplicating the shared data.

The deserializers for shared pointers hold a shared pointer to any deserialized values, and will hold them in memory until the deserializer is dropped. This means that if you serialize only weak pointers to some shared data, they will point to the correct value when deserialized but will point to nothing as soon as the deserializer is dropped.

## 8. Unsized Types

rkyv supports unsized types out of the box and ships with implementations for the most common unsized types (`str` s and slices). Trait objects can also be supported with `rkyv_dyn`, see [Trait Objects](#trait-objects) for more details.

### 8.1. Metadata

The core concept that enables unsized types is metadata. In rust, pointers to types can be different sizes, in contrast with languages like C and C++ where all pointers are the same size. This is important for the concept of sizing, which you may have encountered through rust's [Sized](https://doc.rust-lang.org/std/marker/trait.Sized.html) trait.

Pointers are composed of two pieces: a data address and some metadata. The data address is what most people think of when they think about pointers; it's the location of the pointed-to data. The metadata for a pointer is extra data that's needed to work safely with the data at the pointed-to location. It can be almost anything, or nothing at all for `Sized` types. Pointers with no extra metadata are sometimes called "narrow" pointers, and pointers _with_ metadata are sometimes called "wide" pointers.

> rkyv uses the [`ptr_meta`](https://docs.rs/ptr_meta) crate to perform these conversions safely. In the future, these may be incorporated as [part of the standard library](https://rust-lang.github.io/rfcs/2580-ptr-meta.html).

Fundamentally, the metadata of a pointer exists to provide the program enough information to safely access, drop, and deallocate structures that are pointed to. For slices, the metadata carries the length of the slice, for trait objects it carries the virtual function table (vtable) pointer, and for custom unsized structs it carries the metadata of the single trailing unsized member.

### 8.2. Archived Metadata

For unsized types, the metadata for a type is archived separately from the relative pointer to the data. This mirrors how rust works internally to support archiving shared pointers and other exotic use cases. This does complicate things somewhat, but for most people the metadata archiving process will end up as just filling out a few functions and returning `()`.

> This is definitely one of the more complicated parts of the library, and can be difficult to wrap your head around. Reading the documentation for `ArchiveUnsized` may help you understand how the system works by working through an example.

## 9. Trait Objects

Trait object serialization is supported through the `rkyv_dyn` crate. This crate is maintained as part of rkyv, but is separate from the main crate to allow other implementations to be used instead. This section will focus primarily on the architecture of `rkyv_dyn` and how to use it effectively.

> `rkyv_dyn` may not work in some exotic environments due to the ✨magic✨ it uses to register trait objects. If you want these capabilities but `rkyv_dyn` doesn't work in your environment, feel free to file an issue or drop by in the discord to talk it through.

### 9.1. Core traits

The new traits introduced by `rkyv_dyn` are [`SerializeDyn`](https://docs.rs/rkyv_dyn/latest/rkyv_dyn/trait.SerializeDyn.html) and [`DeserializeDyn`](https://docs.rs/rkyv_dyn/latest/rkyv_dyn/trait.DeserializeDyn.html). These are effectively type-erased versions of `SerializeUnsized` and `DeserializeUnsized` so that the traits are object-safe. Likewise, it introduces type-erased versions of serializers and deserializers: [`DynSerializer`](https://docs.rs/rkyv_dyn/latest/rkyv_dyn/trait.DynSerializer.html) and [`DynDeserializer`](https://docs.rs/rkyv_dyn/latest/rkyv_dyn/trait.DynDeserializer.html). These attempt to provide the basic functionality required to serialize most types, but may be more or less capable than custom types require.

> `DynSerializer` implements the `Serializer` and `ScratchSpace` traits, but that may not be suitable for all use cases. If you need more capabilities, file an issue or drop by in the discord to talk it through.

### 9.2. Architecture

It is highly recommended to use the provided [`archive_dyn`](https://docs.rs/rkyv_dyn/latest/rkyv_dyn/attr.archive_dyn.html) macro to implement the new traits and set everything up correctly.

Using `archive_dyn` on a trait definition creates another trait definition with supertraits of your trait and `SerializeDyn`. This "shim" trait is blanket implemented for all types that implement your trait and `SerializeDyn`, so you should only ever have to implement your trait to use it.

The shim trait should be used everywhere that you have a trait object of your trait that you want to serialize. By default, it will be named "Serialize" + your trait name. A different approach that similar libraries take is directly adding `SerializeDyn` as a supertrait of your trait. While more ergonomic, this approach does not allow the implementation of the trait on types that cannot or should not implement `SerializeDyn`, so the shim trait approach was favored for `rkyv_dyn`.

When the shim trait is serialized, it stores the type hash of the underlying type in its metadata so it can get the correct vtable for it when accessed. This requires that all vtables for implementing types must be known ahead of time, which is when we use `archive_dyn` for the second time.

## 10. Validation

Validation can be enabled with the `bytecheck` feature, and leverages the [`bytecheck`](https://docs.rs/bytecheck) crate to perform archive validation. This allows the use of untrusted and malicious data.

If the `bytecheck` feature is enabled, then rkyv will automatically derive [`CheckBytes`](https://docs.rs/bytecheck/latest/bytecheck/trait.CheckBytes.html) for your archived type:

```rust
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize)]
pub struct Example {
    a: i32,
    b: String,
    c: Vec<bool>,
}
```

The `#[rkyv(bytecheck(..))]` attribute passes its arguments through to the underlying `CheckBytes` derive on the archived type. Finally, you can use `access` to check an archive and get a reference to the archived value if it was successful:

```rust
use rkyv::{access, rancor::Failure};

let archived_example = access::<ArchivedExample, Failure>(buffer).unwrap();
```

### 10.1. The validation context

When checking an archive, a validation context is created automatically using some good defaults that will work for most archived types. If your type requires special validation logic, you may need to augment the capabilities of the validation context in order to check your type.

### 10.2. Bounds checking and subtree ranges

All pointers are checked to make sure that they:

- Point inside the archive
- Are properly aligned
- And have enough space afterward to hold the desired object

However, this alone is not enough to secure against recursion attacks and memory sharing violations, so rkyv uses a system to verify that the archive follows its strict ownership model.

Archive validation uses a memory model where all subobjects are located in contiguous memory. This is called a _subtree range_. When validating an object, the archive context keeps track of where subobjects are allowed to be located, and can reduce the subtree range from the beginning by pushing a new subtree range. After pushing a subtree range, any subobjects in that range can be checked by calling their `CheckBytes` implementations. Once the subobjects are checked, the subtree range can be popped to restore the original range with the checked section removed.

### 10.3. Validation and Shared Pointers

While validating shared pointers is supported, some additional restrictions are in place to prevent malicious data from validating.

Shared pointers that point to the same object will fail to validate if they are different types. This can cause issues if you have a shared pointer to the same array, but the pointers are an array pointer and a slice pointer. Similarly, it can cause issues if you have shared pointers to the same value as a concrete type (e.g. `i32`) and a trait object (e.g. `dyn Any`).

rkyv still supports these use cases, but it's not possible or feasible to ensure data integrity with these use cases. Alternative validation solutions like archive signatures and data hashes may be a better approach in these cases.

## 11. Allocation tracking

rkyv's provided `AllocationTracker` struct wraps an `Allocator` and tracks when memory is allocated and freed during serialization. It can also calculate synthetic metrics, like the minimum amount of pre-allocated memory required to serialize a value. And, it can report the maximum alignment of all serialized types.

You can create a custom serializer with allocation tracking by calling `Serializer::new(..)` and providing the pieces of your serializer. Normally, the provided allocator would be an `ArenaHandle`, but instead you should provide it an `AllocationTracker::new(arena_handle)`.

After serializing your value, the serializer can be decomposed with `into_raw_parts`. You can then retrieve the `AllocationStats` from the allocator by calling `into_stats`.

## 12. Feature Comparison

This is a best-effort feature comparison between rkyv, FlatBuffers, and Cap'n Proto. This is by no means completely comprehensive, and pull requests that improve this are welcomed.

### 12.1. Feature matrix

| Feature                          | rkyv      | Cap'n Proto | FlatBuffers |
| -------------------------------- | --------- | ----------- | ----------- |
| Open type system                 | yes       | no          | no          |
| Scalars                          | yes       | no          | yes         |
| Tables                           | no        | yes         | yes         |
| Schema evolution                 | no        | yes         | yes         |
| Zero-copy                        | yes       | yes         | yes         |
| Random-access reads              | yes       | yes         | yes         |
| Validation                       | upfront   | on-demand   | yes         |
| Reflection                       | no        | yes         | yes         |
| Object order                     | bottom-up | either      | bottom-up   |
| Schema language                  | derive    | custom      | custom      |
| Usable as mutable state          | limited   | limited     | limited     |
| Padding takes space on wire?     | optional  | optional    | no          |
| Unset fields take space on wire? | yes       | yes         | no          |
| Pointers take space on wire?     | yes       | yes         | yes         |
| Cross-language                   | no        | yes         | yes         |
| Hash maps and B-trees            | yes       | no          | no          |
| Shared pointers                  | yes       | no          | no          |

Although these features aren't supported out-of-the-box, rkyv's open type system allows extensions which provide many of these capabilities.

### 12.2. Open type system

One of rkyv's primary features is that its type system is _open_. This means that users can write custom types and control their properties very finely. You can think of rkyv as a solid foundation to build many other features on top of. In fact, the open type system is already a fundamental part of how rkyv works.

#### Unsized types

Even though they're part of the main library, unsized types are built on top of the core serialization functionality. Types like `Box` and `Rc/Arc` that can hold unsized types are entry points for unsized types into the sized system.

#### Trait objects

Trait objects are further built on top of unsized types to make serializing and using trait objects easy and safe.

## 13. FAQ

Because it's so different from traditional serialization systems, a lot of people have questions about rkyv. This is meant to serve as a comprehensive, centralized source for answers.

### How is rkyv zero-copy? It definitely copies the archive into memory.

Traditional serialization works in two steps:

1. Read the data from disk into a buffer (maybe in pieces)
2. Process the data in the buffer into the deserialized data structure

The copy happens in the second step, when the data in the buffer ends up duplicated in the final data structure. Zero-copy deserialization doesn't deserialize the buffer into a separate structure, and thus avoids this copy.

Advanced techniques like memory-mapped files can also help you avoid copying as much of your data if you only read smaller parts of it.

### How does rkyv handle endianness?

rkyv supports little- and big-endian formats. You can enable specific endiannesses with the `little_endian` and `big_endian` features, or default to little-endian byte ordering.

### Is rkyv cross-platform?

Yes.

### Can I use this in embedded and #\[no_std\] environments?

Yes, disable the `std` feature for `no_std`. You can additionally disable the `alloc` feature to disable all memory allocation capabilities.

### Safety

#### Isn't this very unsafe if you access untrusted data?

If you skip validation, then yes. You can still access untrusted data if you validate the archive first with bytecheck. It's an extra step, but it's usually still less than the cost of deserializing using a traditional format.

#### Doesn't that mean I always have to validate?

No, there are many other ways you can verify your data, for example with checksums and cryptographic signatures.

#### Isn't it kind of deceptive to say rkyv is fast and then require validation?

The fastest path to access archived data is marked as `unsafe`. This doesn't mean that it's unusable, it means that it's only safe to call if you can verify its preconditions.

As long as you can reasonably uphold those preconditions, then accessing the archive is safe. Not every archive needs to be validated, and you can use a variety of different techniques to guarantee data integrity and security.

Even if you do need to always validate your data before accessing it, validation is still faster than deserializing with other high-performance formats. A round-trip is still faster, even though it's not by the same margins.

## Contributors

Thanks to all the contributors who have helped document rkyv:

- David Koloski ([djkoloski](https://github.com/djkoloski))
- Badewanne3 ([MaxOhn](https://github.com/MaxOhn))

If you feel you're missing from this list, feel free to add yourself in a PR.
