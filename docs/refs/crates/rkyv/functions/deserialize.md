# `rkyv` Function: `deserialize`

Source: <https://docs.rs/rkyv/latest/rkyv/fn.deserialize.html>

```rust
pub fn deserialize<T, E>(
    value: &impl Deserialize<T, HighDeserializer<E>>,
) -> Result<T, E>
```

Available on **crate feature `alloc`** only.

Deserialize a value from the given archived value.

This is part of the [high-level API](https://docs.rs/rkyv/latest/rkyv/api/high/index.html "mod rkyv::api::high").

## Example

```rust
use rkyv::{
    access, deserialize, rancor::Error, to_bytes, Archive, Deserialize,
    Serialize,
};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
struct Example {
    name: String,
    value: i32,
}

let value = Example {
    name: "pi".to_string(),
    value: 31415926,
};

let bytes = to_bytes::<Error>(&value).unwrap();
let archived = access::<ArchivedExample, Error>(&*bytes).unwrap();
let deserialized = deserialize::<Example, Error>(archived).unwrap();

assert_eq!(deserialized, value);
```
