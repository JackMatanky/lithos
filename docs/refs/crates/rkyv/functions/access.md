# `rkyv` Function: `access`

Source: <https://docs.rs/rkyv/latest/rkyv/fn.access.html>

```rust
pub fn access<T, E>(bytes: &[u8]) -> Result<&T, E>where
    T: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
    E: Source,
```

Available on **crate features `bytecheck` and `alloc`** only.

This is a safe alternative to [`access_unchecked`](https://docs.rs/rkyv/latest/rkyv/fn.access_unchecked.html "fn rkyv::access_unchecked") and is part of the [high-level API](https://docs.rs/rkyv/latest/rkyv/api/high/index.html "mod rkyv::api::high").

## Example

```rust
use rkyv::{
    access, bytecheck::CheckBytes, rancor::Error, to_bytes, Archive,
    Archived, Serialize,
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

let bytes = to_bytes::<Error>(&value).unwrap();
let archived = access::<ArchivedExample, Error>(&bytes).unwrap();

assert_eq!(archived.name, "pi");
assert_eq!(archived.value, 31415926);
```
