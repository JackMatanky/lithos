# redb Advanced Patterns

Source: https://docs.rs/redb/latest/redb/trait.Value.html

## Custom Value Implementation
Implementing the `Value` trait allows using custom types as keys or values.

```rust
use redb::{Value, TypeName};

struct MyType(u64);

impl Value for MyType {
    type SelfType<'a> = MyType;
    type AsBytes<'a> = [u8; 8];

    fn fixed_width() -> Option<usize> {
        Some(8)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> {
        MyType(u64::from_le_bytes(data.try_into().unwrap()))
    }

    fn as_bytes<'a>(value: &'a Self::SelfType<'a>) -> Self::AsBytes<'a> {
        value.0.to_le_bytes()
    }

    fn type_name() -> TypeName {
        TypeName::new("MyType")
    }
}
```

## Multimaps
redb doesn't have a native Multimap type, but it can be implemented by using a key that includes the secondary dimension or by storing a collection as the value.

For efficient multimap-like behavior:
1. Use a composite key: `(Key, SubKey)`.
2. Use `range()` to iterate over all `SubKey`s for a given `Key`.
