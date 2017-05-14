# Humanesort
A crate for sorting the way humans would.

This crate aims to provide the sorting behavior a human might expect.
Say you have a directory of files all called "Something-" with a sequential number appended.
With traditional sorting by character the file "Something-11" would occur after the file
"Something-2".
Often this is not the desired behavior, this crate implements a more human compatible ordering
by treating each occurrence of consecutive digits as a combined number in sorting.

The crate implements the type `HumaneOrder` for common types (currently only most string types) and `HumaneSortable` for slices of
`HumanOrder` types.

The API is very simple to use:

```rust
use humanesort::prelude::*;
let mut sort_me = vec!["something-11", "something-1", "something-2"];
sort_me.humane_sort();
assert_eq!(vec!["something-1", "something-2", "something-11"], sort_me);
```

## Details on String Sorting

For sorting, a string is split into numeric and non-numeric sections.
The comparison starts at the first group and if no group is (by any of the rules) larger than the other
the comparison moves on to the next section. For comparison of sections the following rules are
used.

* Any non-numbers are compared using their usual compare methods
* Numbers are always greater than nun-numbers
* Numeric sequences are ordered by their numeric value
* Empty sequences are always smaller than non-empty ones


These examples should give you some idea of how this works out in practice:

```rust
use humanesort::HumaneSortable;
let mut a = ["lol-1", "lal-2"];
a.humane_sort();
assert_eq!(a, ["lal-2", "lol-1"])
```

```rust
use humanesort::HumaneSortable;
let mut a = ["13-zzzz", "1-ffff", "12-aaaa"];
a.humane_sort();
assert_eq!(a, ["1-ffff", "12-aaaa", "13-zzzz"])
```

