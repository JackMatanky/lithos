# `walkdir` Crate: Struct `FilterEntry`

Source: <https://docs.rs/walkdir/latest/walkdir/struct.FilterEntry.html>

---

```rust
pub struct FilterEntry<I, P> { /* private fields */ }
```

A recursive directory iterator that skips entries.

Values of this type are created by calling [`.filter_entry()`](https://docs.rs/walkdir/latest/walkdir/struct.IntoIter.html#method.filter_entry) on an `IntoIter`, which is formed by calling [`.into_iter()`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#into_iter.v) on a `WalkDir`.

Directories that fail the predicate `P` are skipped. Namely, they are never yielded and never descended into.

Entries that are skipped with the [`min_depth`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.min_depth) and [`max_depth`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.max_depth) options are not passed through this filter.

If opening a handle to a directory resulted in an error, then it is yielded and no corresponding call to the predicate is made.

Type parameter `I` refers to the underlying iterator and `P` refers to the predicate, which is usually `FnMut(&DirEntry) -> bool`.

## Implementations

### `impl<P> FilterEntry<IntoIter, P>`

```rust
impl<P> FilterEntry<IntoIter, P>
where
    P: FnMut(&DirEntry) -> bool,
```

#### `pub fn filter_entry(self, predicate: P) -> FilterEntry<Self, P>`

Yields only entries which satisfy the given predicate and skips descending into directories that do not satisfy the given predicate.

The predicate is applied to all entries. If the predicate is true, iteration carries on as normal. If the predicate is false, the entry is ignored and if it is a directory, it is not descended into.

This is often more convenient to use than [`skip_current_dir`](#method.skip_current_dir). For example, to skip hidden files and directories efficiently on unix systems:

```rust
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

for entry in WalkDir::new("foo")
                     .into_iter()
                     .filter_entry(|e| !is_hidden(e)) {
    println!("{}", entry?.path().display());
}
```

Note that the iterator will still yield errors for reading entries that may not satisfy the predicate.

Note that entries skipped with [`min_depth`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.min_depth) and [`max_depth`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.max_depth) are not passed to this predicate.

Note that if the iterator has `contents_first` enabled, then this method is no different than calling the standard `Iterator::filter` method (because directory entries are yielded after they’ve been descended into).

#### `pub fn skip_current_dir(&mut self)`

Skips the current directory.

This causes the iterator to stop traversing the contents of the least recently yielded directory. This means any remaining entries in that directory will be skipped (including sub-directories).

Note that the ergonomics of this method are questionable since it borrows the iterator mutably. Namely, you must write out the looping condition manually. For example, to skip hidden entries efficiently on unix systems:

```rust
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

let mut it = WalkDir::new("foo").into_iter();
loop {
    let entry = match it.next() {
        None => break,
        Some(Err(err)) => panic!("ERROR: {}", err),
        Some(Ok(entry)) => entry,
    };
    if is_hidden(&entry) {
        if entry.file_type().is_dir() {
            it.skip_current_dir();
        }
        continue;
    }
    println!("{}", entry.path().display());
}
```

You may find it more convenient to use the [`filter_entry`](#method.filter_entry) iterator adapter. (See its documentation for the same example functionality as above.)

## Trait Implementations

### `impl<I: Debug, P: Debug> Debug for FilterEntry<I, P>`

#### `fmt(&self, f: &mut Formatter<'_>) -> Result`

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

### `impl<P> FusedIterator for FilterEntry<IntoIter, P>`

```rust
impl<P> FusedIterator for FilterEntry<IntoIter, P>
where
    P: FnMut(&DirEntry) -> bool,
```

### `impl<P> Iterator for FilterEntry<IntoIter, P>`

```rust
impl<P> Iterator for FilterEntry<IntoIter, P>
where
    P: FnMut(&DirEntry) -> bool,
```

#### `next(&mut self) -> Option<Self::Item>`

Advances the iterator and returns the next value.

##### Errors

If the iterator fails to retrieve the next value, this method returns an error value. The error will be wrapped in an `Option::Some`.

#### `type Item = Result<DirEntry, Error>`

The type of the elements being iterated over.

#### `fn next_chunk<const N: usize>(&mut self) -> Result<[Self::Item; N], IntoIter<Self::Item, N>>`

```rust
fn next_chunk<const N: usize>(
    &mut self,
) -> Result<[Self::Item; N], IntoIter<Self::Item, N>>
where
    Self: Sized,
```

🔬This is a nightly-only experimental API. (`iter_next_chunk`)

Advances the iterator and returns an array containing the next `N` values. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.next_chunk)

#### `fn size_hint(&self) -> (usize, Option<usize>)`

Returns the bounds on the remaining length of the iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.size_hint)

#### `fn count(self) -> usize`

```rust
fn count(self) -> usize
where
    Self: Sized,
```

Consumes the iterator, counting the number of iterations and returning it. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.count)

#### `fn last(self) -> Option<Self::Item>`

```rust
fn last(self) -> Option<Self::Item>
where
    Self: Sized,
```

Consumes the iterator, returning the last element. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.last)

#### `fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>>`

🔬This is a nightly-only experimental API. (`iter_advance_by`)

Advances the iterator by `n` elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.advance_by)

#### `fn nth(&mut self, n: usize) -> Option<Self::Item>`

Returns the `n` th element of the iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.nth)

#### `fn step_by(self, step: usize) -> StepBy<Self>`

```rust
fn step_by(self, step: usize) -> StepBy<Self>
where
    Self: Sized,
```

Creates an iterator starting at the same point, but stepping by the given amount at each iteration. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.step_by)

#### `fn chain<U>(self, other: U) -> Chain<Self, <U as IntoIterator>::IntoIter>`

```rust
fn chain<U>(self, other: U) -> Chain<Self, <U as IntoIterator>::IntoIter>
where
    Self: Sized,
    U: IntoIterator<Item = Self::Item>,
```

Takes two iterators and creates a new iterator over both in sequence. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.chain)

#### `fn zip<U>(self, other: U) -> Zip<Self, <U as IntoIterator>::IntoIter>`

```rust
fn zip<U>(self, other: U) -> Zip<Self, <U as IntoIterator>::IntoIter>
where
    Self: Sized,
    U: IntoIterator,
```

‘Zips up’ two iterators into a single iterator of pairs. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.zip)

#### `fn intersperse_with<G>(self, separator: G) -> IntersperseWith<Self, G>`

```rust
fn intersperse(self, separator: Self::Item) -> Intersperse<Self>
where
    Self: Sized,
    Self::Item: Clone,
```

🔬This is a nightly-only experimental API. (`iter_intersperse`)

Creates a new iterator which places a copy of `separator` between adjacent items of the original iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.intersperse)

#### `fn intersperse_with<G>(self, separator: G) -> IntersperseWith<Self, G>`

```rust
fn intersperse_with<G>(self, separator: G) -> IntersperseWith<Self, G>
where
    Self: Sized,
    G: FnMut() -> Self::Item,
```

🔬This is a nightly-only experimental API. (`iter_intersperse`)

Creates a new iterator which places an item generated by `separator` between adjacent items of the original iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.intersperse_with)

#### `fn map<B, F>(self, f: F) -> Map<Self, F>`

```rust
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    Self: Sized,
    F: FnMut(Self::Item) -> B,
```

Takes a closure and creates an iterator which calls that closure on each element. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.map)

#### `fn for_each<F>(self, f: F)`

```rust
fn for_each<F>(self, f: F)
where
    Self: Sized,
    F: FnMut(Self::Item),
```

Calls a closure on each element of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.for_each)

#### `fn filter<P>(self, predicate: P) -> Filter<Self, P>`

```rust
fn filter<P>(self, predicate: P) -> Filter<Self, P>
where
    Self: Sized,
    P: FnMut(Self::Item) -> bool,
```

Creates an iterator which uses a closure to determine if an element should be yielded. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.filter)

#### `fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>`

```rust
fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>
where
    Self: Sized,
    F: FnMut(Self::Item) -> Option<B>,
```

Creates an iterator that both filters and maps. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.filter_map)

#### `fn enumerate(self) -> Enumerate<Self>`

```rust
fn enumerate(self) -> Enumerate<Self>
where
    Self: Sized,
```

Creates an iterator which gives the current iteration count as well as the next value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.enumerate)

#### `fn peekable(self) -> Peekable<Self>`

```rust
fn peekable(self) -> Peekable<Self>
where
    Self: Sized,
```

Creates an iterator which can use the [`peek`](https://doc.rust-lang.org/nightly/core/iter/adapters/peekable/struct.Peekable.html#method.peek "method core::iter::adapters::peekable::Peekable::peek") and [`peek_mut`](https://doc.rust-lang.org/nightly/core/iter/adapters/peekable/struct.Peekable.html#method.peek_mut "method core::iter::adapters::peekable::Peekable::peek_mut") methods to look at the next element of the iterator without consuming it. See their documentation for more information. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.peekable)

#### `fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>`

```rust
fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
where
    Self: Sized,
    P: FnMut(Self::Item) -> bool,
```

Creates an iterator that [`skip`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.skip "method core::iter::traits::iterator::Iterator::skip") s elements based on a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.skip_while)

#### `fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>`

```rust
fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
where
    Self: Sized,
    P: FnMut(Self::Item) -> bool,
```

Creates an iterator that yields elements based on a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.take_while)

#### `fn map_while<B, P>(self, predicate: P) -> MapWhile<Self, P>`

```rust
fn map_while<B, P>(self, predicate: P) -> MapWhile<Self, P>
where
    Self: Sized,
    P: FnMut(Self::Item) -> Option<B>,
```

Creates an iterator that both yields elements based on a predicate and maps. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.map_while)

#### `fn skip(self, n: usize) -> Skip<Self>`

```rust
fn skip(self, n: usize) -> Skip<Self>
where
    Self: Sized,
```

Creates an iterator that skips the first `n` elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.skip)

#### `fn take(self, n: usize) -> Take<Self>`

```rust
fn take(self, n: usize) -> Take<Self>
where
    Self: Sized,
```

Creates an iterator that yields the first `n` elements, or fewer if the underlying iterator ends sooner. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.take)

#### `fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>`

```rust
fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
where
    Self: Sized,
    F: FnMut(St, Self::Item) -> Option<B>,
```

An iterator adapter which, like [`fold`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fold "method core::iter::traits::iterator::Iterator::fold"), holds internal state, but unlike [`fold`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fold "method core::iter::traits::iterator::Iterator::fold"), produces a new iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.scan)

#### `fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F>`

```rust
fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F>
where
    Self: Sized,
    U: IntoIterator,
    F: FnMut(Self::Item) -> U,
```

Creates an iterator that works like map, but flattens nested structure. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.flat_map)

#### `fn flatten(self) -> Flatten<Self>`

```rust
fn flatten(self) -> Flatten<Self>
where
    Self: Sized,
    Self::Item: IntoIterator,
```

Creates an iterator that flattens nested structure. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.flatten)

#### `fn map_windows<F, R, const N: usize>(self, f: F) -> MapWindows<Self, F, N>`

```rust
fn map_windows<F, R, const N: usize>(self, f: F) -> MapWindows<Self, F, N>
where
    Self: Sized,
    F: FnMut(&[Self::Item; N]) -> R,
```

🔬This is a nightly-only experimental API. (`iter_map_windows`)

Calls the given function `f` for each contiguous window of size `N` over `self` and returns an iterator over the outputs of `f`. Like [`slice::windows()`](https://doc.rust-lang.org/nightly/std/primitive.slice.html#method.windows "method slice::windows"), the windows during mapping overlap as well. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.map_windows)

#### `fn fuse(self) -> Fuse<Self>`

```rust
fn fuse(self) -> Fuse<Self>
where
    Self: Sized,
```

Creates an iterator which ends after the first [`None`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None "variant core::option::Option::None"). [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fuse)

#### `fn inspect<F>(self, f: F) -> Inspect<Self, F>`

```rust
fn inspect<F>(self, f: F) -> Inspect<Self, F>
where
    Self: Sized,
    F: FnMut(Self::Item),
```

Does something with each element of an iterator, passing the value on. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.inspect)

#### `fn by_ref(&mut self) -> &mut Self`

```rust
fn by_ref(&mut self) -> &mut Self
where
    Self: Sized,
```

Creates a "by reference" adapter for this instance of `Iterator`. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.by_ref)

#### `fn collect<B>(self) -> B`

```rust
fn collect<B>(self) -> B
where
    B: FromIterator<Self::Item>,
    Self: Sized,
```

Transforms an iterator into a collection. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.collect)

#### `fn try_collect<B>(&mut self) -> <<Self::Item as Try>::Residual as Residual<B>>::TryType`

```rust
fn try_collect<B>(
    &mut self,
) -> <<Self::Item as Try>::Residual as Residual<B>>::TryType
where
    Self: Sized,
    Self::Item: Try,
    <Self::Item as Try>::Residual: Residual<B>,
    B: FromIterator<<Self::Item as Try>::Output>,
```

🔬This is a nightly-only experimental API. (`iterator_try_collect`)

Fallibly transforms an iterator into a collection, short circuiting if a failure is encountered. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_collect)

#### `fn collect_into<B>(self, collection: &mut B) -> Result<(), B::Error>`

```rust
fn collect_into<E>(self, collection: &mut E) -> &mut E
where
    Self: Sized,
    E: Extend<Self::Item>,
```

🔬This is a nightly-only experimental API. (`iter_collect_into`)

Collects all the items from an iterator into a collection. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.collect_into)

#### `fn partition<B, F>(self, f: F) -> (B, B)`

```rust
fn partition<B, F>(self, f: F) -> (B, B)
where
    Self: Sized,
    B: Default + Extend<Self::Item>,
    F: FnMut(Self::Item) -> bool,
```

Consumes an iterator, creating two collections from it. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.partition)

#### `fn is_partitioned<P>(self, predicate: P) -> bool`

```rust
fn is_partitioned<P>(self, predicate: P) -> bool
where
    Self: Sized,
    P: FnMut(Self::Item) -> bool,
```

🔬This is a nightly-only experimental API. (`iter_is_partitioned`)

Checks if the elements of this iterator are partitioned according to the given predicate, such that all those that return `true` precede all those that return `false`. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_partitioned)

#### `fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R`

```rust
fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
where
    Self: Sized,
    F: FnMut(B, Self::Item) -> R,
    R: Try<Output = B>,
```

An iterator method that applies a function as long as it returns successfully, producing a single, final value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_fold)

#### `fn try_for_each<F, R>(&mut self, f: F) -> R`

```rust
fn try_for_each<F, R>(&mut self, f: F) -> R
where
    Self: Sized,
    F: FnMut(Self::Item) -> R,
    R: Try<Output = ()>,
```

An iterator method that applies a fallible function to each item in the iterator, stopping at the first error and returning that error. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_for_each)

#### `fn fold<B, F>(self, init: B, f: F) -> B`

```rust
fn fold<B, F>(self, init: B, f: F) -> B
where
    Self: Sized,
    F: FnMut(B, Self::Item) -> B,
```

Folds every element into an accumulator by applying an operation, returning the final result. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fold)

#### `fn reduce(self, f: F) -> Option<Self::Item>`

```rust
fn reduce(self, f: F) -> Option<Self::Item>
where
    Self: Sized,
    F: FnMut(Self::Item, Self::Item) -> Self::Item,
```

Reduces the elements to a single one, by repeatedly applying a reducing operation. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.reduce)

#### `fn try_reduce<R>(&mut self, f: impl FnMut(Self::Item, Self::Item) -> R) -> <<R as Try>::Residual as Residual<Option<<R as Try>::Output>>>::TryType`

```rust
fn try_reduce<R>(
    &mut self,
    f: impl FnMut(Self::Item, Self::Item) -> R,
) -> <<R as Try>::Residual as Residual<Option<<R as Try>::Output>>>::TryType
where
    Self: Sized,
    R: Try<Output = Self::Item>,
    <R as Try>::Residual: Residual<Option<Self::Item>>,
```

🔬This is a nightly-only experimental API. (`iterator_try_reduce`)

Reduces the elements to a single one by repeatedly applying a reducing operation. If the closure returns a failure, the failure is propagated back to the caller immediately. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_reduce)

#### `fn all<F>(&mut self, f: F) -> bool`

```rust
fn all<F>(&mut self, f: F) -> bool
where
    Self: Sized,
    F: FnMut(Self::Item) -> bool,
```

Tests if every element of the iterator matches a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.all)

#### `fn any<F>(&mut self, f: F) -> bool`

```rust
fn any<F>(&mut self, f: F) -> bool
where
    Self: Sized,
    F: FnMut(Self::Item) -> bool,
```

Tests if any element of the iterator matches a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.any)

#### `fn find<P>(&mut self, predicate: P) -> Option<Self::Item>`

```rust
fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
where
    Self: Sized,
    P: FnMut(&Self::Item) -> bool,
```

Searches for an element of an iterator that satisfies a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.find)

#### `fn find_map<B, F>(&mut self, f: F) -> Option<B>`

```rust
fn find_map<B, F>(&mut self, f: F) -> Option<B>
where
    Self: Sized,
    F: FnMut(Self::Item) -> Option<B>,
```

Applies function to the elements of iterator and returns the first non-none result. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.find_map)

```rust
fn try_find<R>(
    &mut self,
    f: impl FnMut(&Self::Item) -> R,
) -> <<R as Try>::Residual as Residual<Option<Self::Item>>>::TryType
where
    Self: Sized,
    R: Try<Output = bool>,
    <R as Try>::Residual: Residual<Option<Self::Item>>,
```

🔬This is a nightly-only experimental API. (`try_find`)

Applies function to the elements of iterator and returns the first true result or the first error. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_find)

```rust
fn position<P>(&mut self, predicate: P) -> Option<usize>
where
    Self: Sized,
    P: FnMut(Self::Item) -> bool,
```

Searches for an element in an iterator, returning its index. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.position)

#### `fn max(self) -> Option<Self::Item>`

```rust
fn max(self) -> Option<Self::Item>
where
    Self: Sized,
    Self::Item: Ord,
```

Returns the maximum element of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.max)

#### `fn min(self) -> Option<Self::Item>`

```rust
fn min(self) -> Option<Self::Item>
where
    Self: Sized,
    Self::Item: Ord,
```

Returns the minimum element of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.min)

#### `fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>`

```rust
fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>
where
    B: Ord,
    Self: Sized,
    F: FnMut(&Self::Item) -> B,
```

Returns the element that gives the maximum value from the specified function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.max_by_key)

#### `fn max_by<F>(self, compare: F) -> Option<Self::Item>`

```rust
fn max_by<F>(self, compare: F) -> Option<Self::Item>
where
    Self: Sized,
    F: FnMut(&Self::Item, &Self::Item) -> Ordering,
```

Returns the element that gives the maximum value with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.max_by)

#### `fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>`

```rust
fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>
where
    B: Ord,
    Self: Sized,
    F: FnMut(&Self::Item) -> B,
```

Returns the element that gives the minimum value from the specified function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.min_by_key)

#### `fn min_by<F>(self, compare: F) -> Option<Self::Item>`

```rust
fn min_by<F>(self, compare: F) -> Option<Self::Item>
where
    Self: Sized,
    F: FnMut(&Self::Item, &Self::Item) -> Ordering,
```

Returns the element that gives the minimum value with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.min_by)

#### `fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB)`

```rust
fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB)
where
    FromA: Default + Extend<A>,
    FromB: Default + Extend<B>,
    Self: Sized + Iterator<Item = (A, B)>,
```

Converts an iterator of pairs into a pair of containers. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.unzip)

#### `fn copied<'a, T>(self) -> Copied<Self>`

```rust
fn copied<'a, T>(self) -> Copied<Self>
where
    T: Copy + 'a,
    Self: Sized + Iterator<Item = &'a T>,
```

Creates an iterator which copies all of its elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.copied)

#### `fn cloned<'a, T>(self) -> Cloned<Self>`

```rust
fn cloned<'a, T>(self) -> Cloned<Self>
where
    T: Clone + 'a,
    Self: Sized + Iterator<Item = &'a T>,
```

Creates an iterator which [`clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone "method core::clone::Clone::clone") s all of its elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.cloned)

#### `fn array_chunks<const N: usize>(self) -> ArrayChunks<Self, N>`

```rust
fn array_chunks<const N: usize>(self) -> ArrayChunks<Self, N>
where
    Self: Sized,
```

🔬This is a nightly-only experimental API. (`iter_array_chunks`)

Returns an iterator over `N` elements of the iterator at a time. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.array_chunks)

#### `fn sum<S>(self) -> S`

```rust
fn sum<S>(self) -> S
where
    Self: Sized,
    S: Sum<Self::Item>,
```

Sums the elements of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.sum)

#### `fn product<P>(self) -> P`

```rust
fn product<P>(self) -> P
where
    Self: Sized,
    P: Product<Self::Item>,
```

Iterates over the entire iterator, multiplying all the elements [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.product)

#### `fn cmp<I>(self, other: I) -> Ordering`

```rust
fn cmp<I>(self, other: I) -> Ordering
where
    I: IntoIterator<Item = Self::Item>,
    Self::Item: Ord,
    Self: Sized,
```

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.cmp)

#### `fn cmp_by<I, F>(self, other: I, cmp: F) -> Ordering`

```rust
fn cmp_by<I, F>(self, other: I, cmp: F) -> Ordering
where
    Self: Sized,
    I: IntoIterator,
    F: FnMut(Self::Item, <I as IntoIterator>::Item) -> Ordering,
```

🔬This is a nightly-only experimental API. (`iter_order_by`)

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.cmp_by)

#### `fn partial_cmp<I>(self, other: I) -> Option<Ordering>`

```rust
fn partial_cmp<I>(self, other: I) -> Option<Ordering>
where
    I: IntoIterator,
    Self::Item: PartialOrd<<I as IntoIterator>::Item>,
    Self: Sized,
```

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the [`PartialOrd`](https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd") elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another. The comparison works like short-circuit evaluation, returning a result without comparing the remaining elements. As soon as an order can be determined, the evaluation stops and a result is returned. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.partial_cmp)

#### `fn partial_cmp_by<I, F>(self, other: I, partial_cmp: F) -> Option<Ordering>`

```rust
fn partial_cmp_by<I, F>(self, other: I, partial_cmp: F) -> Option<Ordering>
where
    Self: Sized,
    I: IntoIterator,
    F: FnMut(Self::Item, <I as IntoIterator>::Item) -> Option<Ordering>,
```

🔬This is a nightly-only experimental API. (`iter_order_by`)

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.partial_cmp_by)

#### `fn eq<I>(self, other: I) -> bool`

```rust
fn eq<I>(self, other: I) -> bool
where
    I: IntoIterator,
    Self::Item: PartialEq<<I as IntoIterator>::Item>,
    Self: Sized,
```

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.eq)

#### `fn eq_by<I, F>(self, other: I, eq: F) -> bool`

```rust
fn eq_by<I, F>(self, other: I, eq: F) -> bool
where
    Self: Sized,
    I: IntoIterator,
    F: FnMut(Self::Item, <I as IntoIterator>::Item) -> bool,
```

🔬This is a nightly-only experimental API. (`iter_order_by`)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are equal to those of another with respect to the specified equality function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.eq_by)

#### `fn ne<I>(self, other: I) -> bool`

```rust
fn ne<I>(self, other: I) -> bool
where
    I: IntoIterator,
    Self::Item: PartialEq<<I as IntoIterator>::Item>,
    Self: Sized,
```

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are not equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.ne)

#### `fn lt<I>(self, other: I) -> bool`

```rust
fn lt<I>(self, other: I) -> bool
where
    I: IntoIterator,
    Self::Item: PartialOrd<<I as IntoIterator>::Item>,
    Self: Sized,
```

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") less than those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.lt)

#### `fn le<I>(self, other: I) -> bool`

```rust
fn le<I>(self, other: I) -> bool
where
    I: IntoIterator,
    Self::Item: PartialOrd<<I as IntoIterator>::Item>,
    Self: Sized,
```

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") less or equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.le)

#### `fn gt<I>(self, other: I) -> bool`

```rust
fn gt<I>(self, other: I) -> bool
where
    I: IntoIterator,
    Self::Item: PartialOrd<<I as IntoIterator>::Item>,
    Self: Sized,
```

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") greater than those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.gt)

#### `fn ge<I>(self, other: I) -> bool`

```rust
fn ge<I>(self, other: I) -> bool
where
    I: IntoIterator,
    Self::Item: PartialOrd<<I as IntoIterator>::Item>,
    Self: Sized,
```

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") greater than or equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.ge)

#### `fn is_sorted(self) -> bool`

```rust
fn is_sorted(self) -> bool
where
    Self: Sized,
    Self::Item: PartialOrd,
```

Checks if the elements of this iterator are sorted. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_sorted)

#### `fn is_sorted_by<F>(self, compare: F) -> bool`

```rust
fn is_sorted_by<F>(self, compare: F) -> bool
where
    Self: Sized,
    F: FnMut(&Self::Item, &Self::Item) -> bool,
```

Checks if the elements of this iterator are sorted using the given comparator function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_sorted_by)

#### `fn is_sorted_by_key<F, K>(self, f: F) -> bool`

```rust
fn is_sorted_by_key<F, K>(self, f: F) -> bool
where
    Self: Sized,
    F: FnMut(Self::Item) -> K,
    K: PartialOrd,
```

Checks if the elements of this iterator are sorted using the given key extraction function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_sorted_by_key)

## Auto Trait Implementations

### `impl<I, P> Freeze for FilterEntry<I, P>`

```rust
impl<I, P> Freeze for FilterEntry<I, P>
where
    I: Freeze,
    P: Freeze,
```

### `impl<I, P> RefUnwindSafe for FilterEntry<I, P>`

```rust
impl<I, P> RefUnwindSafe for FilterEntry<I, P>
where
    I: RefUnwindSafe,
    P: RefUnwindSafe,
```

### `impl<I, P> Send for FilterEntry<I, P>`

```rust
impl<I, P> Send for FilterEntry<I, P>
where
    I: Send,
    P: Send,
```

### `impl<I, P> Sync for FilterEntry<I, P>`

```rust
impl<I, P> Sync for FilterEntry<I, P>
where
    I: Sync,
    P: Sync,
```

### `impl<I, P> Unpin for FilterEntry<I, P>`

```rust
impl<I, P> Unpin for FilterEntry<I, P>
where
    I: Unpin,
    P: Unpin,
```

### `impl<I, P> UnsafeUnpin for FilterEntry<I, P>`

```rust
impl<I, P> UnsafeUnpin for FilterEntry<I, P>
where
    I: UnsafeUnpin,
    P: UnsafeUnpin,
```

### `impl<I, P> UnwindSafe for FilterEntry<I, P>`

```rust
impl<I, P> UnwindSafe for FilterEntry<I, P>
where
    I: UnwindSafe,
    P: UnwindSafe,
```

## Blanket Implementations

### `impl<T> Any for T`

```rust
impl<T> Any for T
where
    T: 'static + ?Sized,
```

#### `fn type_id(&self) -> TypeId`

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

### `impl<T> Borrow<T> for T`

```rust
impl<T> Borrow<T> for T
where
    T: ?Sized,
```

#### `fn borrow(&self) -> &T`

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

### `impl<T> BorrowMut<T> for T`

```rust
impl<T> BorrowMut<T> for T
where
    T: ?Sized,
```

#### `fn borrow_mut(&mut self) -> &mut T`

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

### `impl<T> From<T> for T`

```rust
impl<T> From<T> for T
```

#### `fn from(t: T) -> T`

Returns the argument unchanged.

### `impl<T, U> Into<U> for T`

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
```

#### `fn into(self) -> U`

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `[From](https://doc.rust-lang.org/nightly/core/convert/trait.From.html "trait core::convert::From")<T> for U` chooses to do.

### `impl<I> IntoIterator for I`

```rust
impl<I> IntoIterator for I
where
    I: Iterator,
```

#### `type Item = <I as Iterator>::Item`

The type of the elements being iterated over.

#### `type IntoIter = I`

Which kind of iterator are we turning this into?

#### `fn into_iter(self) -> I`

Creates an iterator from a value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.IntoIterator.html#tymethod.into_iter)

### `impl<T, U> TryFrom<U> for T`

```rust
impl<T, U> TryFrom<U> for T
where
    U: Into<T>,
```

#### `type Error = Infallible`

The type returned in the event of a conversion error.

#### `fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>`

Performs the conversion.

### `impl<T, U> TryInto<U> for T`

```rust
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
```

#### `type Error = <U as TryFrom<T>>::Error`

The type returned in the event of a conversion error.

#### `fn try_into(self) -> Result<U, <U as TryFrom<T>>::Error>`

Performs the conversion.
