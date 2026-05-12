# `walkdir` Crate: Struct `FilterEntry`

[Source](https://docs.rs/walkdir/latest/walkdir/struct.FilterEntry.html)

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

### impl<P> FilterEntry<IntoIter, P>

#### pub fn (self, predicate: P) -> FilterEntry<Self, P>

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

#### pub fn (&mut self)

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

[Source](https://docs.rs/walkdir/latest/src/walkdir/lib.rs.html#1054)

### impl<I: Debug, P: Debug> Debug for FilterEntry<I, P>

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

### impl<P> Iterator for FilterEntry<IntoIter, P>

Advances the iterator and returns the next value.

##### Errors

If the iterator fails to retrieve the next value, this method returns an error value. The error will be wrapped in an `Option::Some`.

The type of the elements being iterated over.

🔬This is a nightly-only experimental API. (`iter_next_chunk`)

Advances the iterator and returns an array containing the next `N` values. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.next_chunk)

Returns the bounds on the remaining length of the iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.size_hint)

#### fn count(self) -> usizewhere Self: Sized,

Consumes the iterator, counting the number of iterations and returning it. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.count)

Consumes the iterator, returning the last element. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.last)

🔬This is a nightly-only experimental API. (`iter_advance_by`)

Advances the iterator by `n` elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.advance_by)

Returns the `n` th element of the iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.nth)

#### fn step_by(self, step: usize) -> StepBy<Self>where Self: Sized,

Creates an iterator starting at the same point, but stepping by the given amount at each iteration. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.step_by)

#### fn chain<U>(self, other: U) -> Chain<Self, <U as IntoIterator>::IntoIter>where Self: Sized, U: IntoIterator<Item = Self::Item>,

Takes two iterators and creates a new iterator over both in sequence. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.chain)

#### fn zip<U>(self, other: U) -> Zip<Self, <U as IntoIterator>::IntoIter>where Self: Sized, U: IntoIterator,

‘Zips up’ two iterators into a single iterator of pairs. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.zip)

🔬This is a nightly-only experimental API. (`iter_intersperse`)

Creates a new iterator which places a copy of `separator` between adjacent items of the original iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.intersperse)

#### fn intersperse_with<G>(self, separator: G) -> IntersperseWith<Self, G>where Self: Sized, G: FnMut() -> Self::Item,

🔬This is a nightly-only experimental API. (`iter_intersperse`)

Creates a new iterator which places an item generated by `separator` between adjacent items of the original iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.intersperse_with)

#### fn map<B, F>(self, f: F) -> Map<Self, F>where Self: Sized, F: FnMut(Self::Item) -> B,

Takes a closure and creates an iterator which calls that closure on each element. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.map)

#### fn for_each<F>(self, f: F)

Calls a closure on each element of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.for_each)

#### fn filter<P>(self, predicate: P) -> Filter<Self, P>

Creates an iterator which uses a closure to determine if an element should be yielded. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.filter)

#### fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F>

Creates an iterator that both filters and maps. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.filter_map)

#### fn enumerate(self) -> Enumerate<Self>where Self: Sized,

Creates an iterator which gives the current iteration count as well as the next value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.enumerate)

#### fn peekable(self) -> Peekable<Self>where Self: Sized,

Creates an iterator which can use the [`peek`](https://doc.rust-lang.org/nightly/core/iter/adapters/peekable/struct.Peekable.html#method.peek "method core::iter::adapters::peekable::Peekable::peek") and [`peek_mut`](https://doc.rust-lang.org/nightly/core/iter/adapters/peekable/struct.Peekable.html#method.peek_mut "method core::iter::adapters::peekable::Peekable::peek_mut") methods to look at the next element of the iterator without consuming it. See their documentation for more information. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.peekable)

#### fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>

Creates an iterator that [`skip`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.skip "method core::iter::traits::iterator::Iterator::skip") s elements based on a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.skip_while)

#### fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>

Creates an iterator that yields elements based on a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.take_while)

#### fn map_while<B, P>(self, predicate: P) -> MapWhile<Self, P>

Creates an iterator that both yields elements based on a predicate and maps. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.map_while)

#### fn skip(self, n: usize) -> Skip<Self>where Self: Sized,

Creates an iterator that skips the first `n` elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.skip)

#### fn take(self, n: usize) -> Take<Self>where Self: Sized,

Creates an iterator that yields the first `n` elements, or fewer if the underlying iterator ends sooner. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.take)

#### fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>

An iterator adapter which, like [`fold`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fold "method core::iter::traits::iterator::Iterator::fold"), holds internal state, but unlike [`fold`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fold "method core::iter::traits::iterator::Iterator::fold"), produces a new iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.scan)

#### fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F>where Self: Sized, U: IntoIterator, F: FnMut(Self::Item) -> U,

Creates an iterator that works like map, but flattens nested structure. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.flat_map)

Creates an iterator that flattens nested structure. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.flatten)

#### fn map_windows<F, R, const N: usize>(self, f: F) -> MapWindows<Self, F, N>where Self: Sized, F: FnMut(&\[Self::Item; N\]) -> R,

🔬This is a nightly-only experimental API. (`iter_map_windows`)

Calls the given function `f` for each contiguous window of size `N` over `self` and returns an iterator over the outputs of `f`. Like [`slice::windows()`](https://doc.rust-lang.org/nightly/std/primitive.slice.html#method.windows "method slice::windows"), the windows during mapping overlap as well. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.map_windows)

#### fn fuse(self) -> Fuse<Self>where Self: Sized,

Creates an iterator which ends after the first [`None`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None "variant core::option::Option::None"). [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fuse)

#### fn inspect<F>(self, f: F) -> Inspect<Self, F>

Does something with each element of an iterator, passing the value on. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.inspect)

#### fn by_ref(&mut self) -> &mut Selfwhere Self: Sized,

Creates a “by reference” adapter for this instance of `Iterator`. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.by_ref)

#### fn collect<B>(self) -> B

Transforms an iterator into a collection. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.collect)

🔬This is a nightly-only experimental API. (`iterator_try_collect`)

Fallibly transforms an iterator into a collection, short circuiting if a failure is encountered. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_collect)

🔬This is a nightly-only experimental API. (`iter_collect_into`)

Collects all the items from an iterator into a collection. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.collect_into)

#### fn partition<B, F>(self, f: F) -> (B, B)

Consumes an iterator, creating two collections from it. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.partition)

🔬This is a nightly-only experimental API. (`iter_is_partitioned`)

Checks if the elements of this iterator are partitioned according to the given predicate, such that all those that return `true` precede all those that return `false`. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_partitioned)

#### fn try_fold<B, F, R>(&mut self, init: B, f: F) -> Rwhere Self: Sized, F: FnMut(B, Self::Item) -> R, R: Try<Output = B>,

An iterator method that applies a function as long as it returns successfully, producing a single, final value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_fold)

#### fn try_for_each<F, R>(&mut self, f: F) -> Rwhere Self: Sized, F: FnMut(Self::Item) -> R, R: Try<Output = ()>,

An iterator method that applies a fallible function to each item in the iterator, stopping at the first error and returning that error. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_for_each)

#### fn fold<B, F>(self, init: B, f: F) -> Bwhere Self: Sized, F: FnMut(B, Self::Item) -> B,

Folds every element into an accumulator by applying an operation, returning the final result. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.fold)

Reduces the elements to a single one, by repeatedly applying a reducing operation. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.reduce)

🔬This is a nightly-only experimental API. (`iterator_try_reduce`)

Reduces the elements to a single one by repeatedly applying a reducing operation. If the closure returns a failure, the failure is propagated back to the caller immediately. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_reduce)

#### fn all<F>(&mut self, f: F) -> bool

Tests if every element of the iterator matches a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.all)

#### fn any<F>(&mut self, f: F) -> bool

Tests if any element of the iterator matches a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.any)

Searches for an element of an iterator that satisfies a predicate. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.find)

#### fn find_map<B, F>(&mut self, f: F) -> Option<B>

Applies function to the elements of iterator and returns the first non-none result. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.find_map)

🔬This is a nightly-only experimental API. (`try_find`)

Applies function to the elements of iterator and returns the first true result or the first error. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.try_find)

Searches for an element in an iterator, returning its index. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.position)

Returns the maximum element of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.max)

Returns the minimum element of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.min)

#### fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>where B: Ord, Self: Sized, F: FnMut(&Self::Item) -> B,

Returns the element that gives the maximum value from the specified function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.max_by_key)

Returns the element that gives the maximum value with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.max_by)

#### fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>where B: Ord, Self: Sized, F: FnMut(&Self::Item) -> B,

Returns the element that gives the minimum value from the specified function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.min_by_key)

Returns the element that gives the minimum value with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.min_by)

#### fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB)where FromA: Default + Extend<A>, FromB: Default + Extend<B>, Self: Sized + Iterator<Item = (A, B)>,

Converts an iterator of pairs into a pair of containers. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.unzip)

#### fn copied<'a, T>(self) -> Copied<Self>where T: Copy + 'a, Self: Sized + Iterator<Item = &'a T>,

Creates an iterator which copies all of its elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.copied)

#### fn cloned<'a, T>(self) -> Cloned<Self>where T: Clone + 'a, Self: Sized + Iterator<Item = &'a T>,

Creates an iterator which [`clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone "method core::clone::Clone::clone") s all of its elements. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.cloned)

🔬This is a nightly-only experimental API. (`iter_array_chunks`)

Returns an iterator over `N` elements of the iterator at a time. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.array_chunks)

#### fn sum<S>(self) -> S

Sums the elements of an iterator. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.sum)

#### fn product<P>(self) -> P

Iterates over the entire iterator, multiplying all the elements [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.product)

#### fn cmp<I>(self, other: I) -> Ordering

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.cmp)

#### fn cmp_by<I, F>(self, other: I, cmp: F) -> Ordering

🔬This is a nightly-only experimental API. (`iter_order_by`)

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.cmp_by)

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the [`PartialOrd`](https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd") elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another. The comparison works like short-circuit evaluation, returning a result without comparing the remaining elements. As soon as an order can be determined, the evaluation stops and a result is returned. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.partial_cmp)

🔬This is a nightly-only experimental API. (`iter_order_by`)

[Lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") compares the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") with those of another with respect to the specified comparison function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.partial_cmp_by)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.eq)

#### fn eq_by<I, F>(self, other: I, eq: F) -> bool

🔬This is a nightly-only experimental API. (`iter_order_by`)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are equal to those of another with respect to the specified equality function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.eq_by)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are not equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.ne)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") less than those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.lt)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") less or equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.le)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") greater than those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.gt)

Determines if the elements of this [`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator") are [lexicographically](https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#lexicographical-comparison "trait core::cmp::Ord") greater than or equal to those of another. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.ge)

Checks if the elements of this iterator are sorted. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_sorted)

Checks if the elements of this iterator are sorted using the given comparator function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_sorted_by)

#### fn is_sorted_by_key<F, K>(self, f: F) -> boolwhere Self: Sized, F: FnMut(Self::Item) -> K, K: PartialOrd,

Checks if the elements of this iterator are sorted using the given key extraction function. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#method.is_sorted_by_key)

### impl<P> FusedIterator for FilterEntry<IntoIter, P>

## Auto Trait Implementations

### impl<I, P> Freeze for FilterEntry<I, P>where I: Freeze, P: Freeze,

### impl<I, P> RefUnwindSafe for FilterEntry<I, P>where I: RefUnwindSafe, P: RefUnwindSafe,

### impl<I, P> Send for FilterEntry<I, P>where I: Send, P: Send,

### impl<I, P> Sync for FilterEntry<I, P>where I: Sync, P: Sync,

### impl<I, P> Unpin for FilterEntry<I, P>where I: Unpin, P: Unpin,

### impl<I, P> UnwindSafe for FilterEntry<I, P>where I: UnwindSafe, P: UnwindSafe,

## Blanket Implementations

[Source](https://doc.rust-lang.org/nightly/src/core/any.rs.html#138)

### impl<T> Any for Twhere T: 'static +?Sized,

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#212)

### impl<T> Borrow<T> for Twhere T:?Sized,

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#221)

### impl<T> BorrowMut<T> for Twhere T:?Sized,

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#222)

#### fn borrow_mut(&mut self) -> &mut T

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#785)

### impl<T> From<T> for T

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#788)

#### fn from(t: T) -> T

Returns the argument unchanged.

### impl<T, U> Into<U> for Twhere U: From<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#777)

#### fn into(self) -> U

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `From<T> for U` chooses to do.

[Source](https://doc.rust-lang.org/nightly/src/core/iter/traits/collect.rs.html#314)

### impl<I> IntoIterator for Iwhere I: Iterator,

The type of the elements being iterated over.

[Source](https://doc.rust-lang.org/nightly/src/core/iter/traits/collect.rs.html#316)

#### type IntoIter = I

Which kind of iterator are we turning this into?

[Source](https://doc.rust-lang.org/nightly/src/core/iter/traits/collect.rs.html#319)

#### fn into_iter(self) -> I

Creates an iterator from a value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.IntoIterator.html#tymethod.into_iter)

### impl<T, U> TryFrom<U> for Twhere U: Into<T>,

The type returned in the event of a conversion error.

Performs the conversion.

### impl<T, U> TryInto<U> for Twhere U: TryFrom<T>,

The type returned in the event of a conversion error.

Performs the conversion.
