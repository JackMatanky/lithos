# `rkyv` Function: `access_mut`

Source: <https://docs.rs/rkyv/latest/rkyv/fn.access_mut.html>

```rust
pub fn access_mut<T, E>(bytes: &mut [u8]) -> Result<Seal<'_, T>, E>where
    T: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
    E: Source,
```

Available on **crate features `bytecheck` and `alloc`** only.

This is a safe alternative to [`access_unchecked_mut`](https://docs.rs/rkyv/latest/rkyv/fn.access_unchecked_mut.html "fn rkyv::access_unchecked_mut") and is part of the [high-level API](https://docs.rs/rkyv/latest/rkyv/api/high/index.html "mod rkyv::api::high").

## Example

```rust
use rkyv::{
    access_mut,
    bytecheck::CheckBytes,
    rancor::Error, munge::munge,
    to_bytes, Archive, Archived, Serialize,
};

#[derive(Archive, Serialize)]
struct Example {
    name: String,
    value: i32,
}

let value = Example {
    name: "pi".to_string(),
    value: 31415926,
};

let mut bytes = to_bytes::<Error>(&value).unwrap();

let mut archived = access_mut::<ArchivedExample, Error>(&mut bytes)
    .unwrap();

// Because the access is mutable, we can mutate the archived data
munge!(let ArchivedExample { mut value, .. } = archived);
assert_eq!(*value, 31415926);
*value = 12345.into();
assert_eq!(*value, 12345);
```
