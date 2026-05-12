# `std::fs`: `DirEntry`

[Source](https://doc.rust-lang.org/std/fs/struct.DirEntry.html)

```rust
pub struct DirEntry(/* private fields */);
```

Entries returned by the [`ReadDir`](https://doc.rust-lang.org/std/fs/struct.ReadDir.html "struct std::fs::ReadDir") iterator.

An instance of `DirEntry` represents an entry inside of a directory on the filesystem. Each entry can be inspected via methods to learn about the full path or possibly other metadata through per-platform extension traits.

## Platform-specific behavior

On Unix, the `DirEntry` struct contains an internal reference to the open directory. Holding `DirEntry` objects will consume a file handle even after the `ReadDir` iterator is dropped.

Note that this [may change in the future](https://doc.rust-lang.org/std/io/index.html#platform-specific-behavior "mod std::io").

## Implementations

### impl DirEntry

#### `pub fn (&self) -> PathBuf`

Returns the full path to the file that this entry represents.

The full path is created by joining the original path to `read_dir` with the filename of this entry.

##### Examples

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    for entry in fs::read_dir(".")? {
        let dir = entry?;
        println!("{:?}", dir.path());
    }
    Ok(())
}
```

This prints output like:

```
"./whatever.txt"
"./foo.html"
"./hello_world.rs"
```

The exact text, of course, depends on what files you have in `.`.

Returns the metadata for the file that this entry points at.

This function will not traverse symlinks if this entry points at a symlink. To traverse symlinks use [`fs::metadata`](https://doc.rust-lang.org/std/fs/fn.metadata.html "fn std::fs::metadata") or [`fs::File::metadata`](https://doc.rust-lang.org/std/fs/struct.File.html#method.metadata "method std::fs::File::metadata").

##### Platform-specific behavior

On Windows this function is cheap to call (no extra system calls needed), but on Unix platforms this function is the equivalent of calling `symlink_metadata` on the path.

##### Examples

```rust
use std::fs;

if let Ok(entries) = fs::read_dir(".") {
    for entry in entries {
        if let Ok(entry) = entry {
            // Here, \`entry\` is a \`DirEntry\`.
            if let Ok(metadata) = entry.metadata() {
                // Now let's show our entry's permissions!
                println!("{:?}: {:?}", entry.path(), metadata.permissions());
            } else {
                println!("Couldn't get metadata for {:?}", entry.path());
            }
        }
    }
}
```

Returns the file type for the file that this entry points at.

This function will not traverse symlinks if this entry points at a symlink.

##### Platform-specific behavior

On Windows and most Unix platforms this function is free (no extra system calls needed), but some Unix platforms may require the equivalent call to `symlink_metadata` to learn about the target file type.

##### Examples

```rust
use std::fs;

if let Ok(entries) = fs::read_dir(".") {
    for entry in entries {
        if let Ok(entry) = entry {
            // Here, \`entry\` is a \`DirEntry\`.
            if let Ok(file_type) = entry.file_type() {
                // Now let's show our entry's file type!
                println!("{:?}: {:?}", entry.path(), file_type);
            } else {
                println!("Couldn't get file type for {:?}", entry.path());
            }
        }
    }
}
```

#### `pub fn (&self) -> OsString`

Returns the file name of this directory entry without any leading path component(s).

As an example, the output of the function will result in “foo” for all the following paths:

- “./foo”
- “/the/foo”
- “../../foo”

##### Examples

```rust
use std::fs;

if let Ok(entries) = fs::read_dir(".") {
    for entry in entries {
        if let Ok(entry) = entry {
            // Here, \`entry\` is a \`DirEntry\`.
            println!("{:?}", entry.file_name());
        }
    }
}
```

## Trait Implementations

### `impl Debug for DirEntry`

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/std/fmt/trait.Debug.html#tymethod.fmt)

### `impl DirEntryExt for DirEntry`

Available on **Unix** only.

Returns the underlying `d_ino` field in the contained `dirent` structure. [Read more](https://doc.rust-lang.org/std/os/unix/fs/trait.DirEntryExt.html#tymethod.ino)

### `impl DirEntryExt for DirEntry`

Available on **WASI** only.

🔬This is a nightly-only experimental API. (`wasi_ext` [#71213](https://github.com/rust-lang/rust/issues/71213))

Returns the underlying `d_ino` field of the `dirent_t`

### `impl DirEntryExt2 for DirEntry`

Available on **Unix** only.

🔬This is a nightly-only experimental API. (`dir_entry_ext2` [#85573](https://github.com/rust-lang/rust/issues/85573))

Returns a reference to the underlying `OsStr` of this entry’s filename. [Read more](https://doc.rust-lang.org/std/os/unix/fs/trait.DirEntryExt2.html#tymethod.file_name_ref)

## Auto Trait Implementations

## Blanket Implementations

```rust
impl<T> Any for T
    where T: 'static + ?Sized,

fn type_id(&self) -> TypeId
```

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/std/any/trait.Any.html#tymethod.type_id)

---

```rust
impl<T> Borrow<T> for T
    where T: ?Sized,

fn borrow(&self) -> &T
```

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/std/borrow/trait.Borrow.html#tymethod.borrow)

---

```rust
impl<T> BorrowMut<T> for T
    where T: ?Sized,

fn borrow_mut(&mut self) -> &mut T
```

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/std/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

---

```rust
impl<T> From<T> for T

fn from(t: T) -> T
```

Returns the argument unchanged.

---

```rust
impl<T, U> Into<U> for T
    where U: From<T>,

fn into(self) -> U
```

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `From<T> for U` chooses to do.

---

```rust
impl<T, U> TryFrom<U> for T
    where U: Into<T>,

fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>
```

The type returned in the event of a conversion error.

Performs the conversion.

---

```rust
impl<T, U> TryInto<U> for T
    where U: TryFrom<T>,

fn try_into(self) -> Result<U, <U as TryFrom<T>>::Error>
```

The type returned in the event of a conversion error.

Performs the conversion.
