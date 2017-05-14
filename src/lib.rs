//! A crate for sorting the way humans would.
//!
//! This crate aims to provide the sorting behavior a human might expect.
//! Say you have a directory of files all called "Something-" with a sequential number appended.
//! With traditional sorting by character the file "Something-11" would occur after the file
//! "Something-2".
//! Often this is not the desired behavior, this crate implements a more human compatible ordering
//! by treating each occurrence of consecutive digits as a combined number in sorting.
//!
//! The crate implements the type `HumaneOrder` for common types (currently only most string types) and `HumaneSortable` for slices of
//! `HumanOrder` types.
//!
//! The API is very simple to use:
//!
//! ```
//! use humanesort::prelude::*;
//! let mut sort_me = vec!["something-11", "something-1", "something-2"];
//! sort_me.humane_sort();
//! assert_eq!(vec!["something-1", "something-2", "something-11"], sort_me);
//! ```
//!
//! ## Details on String Sorting
//!
//! For sorting, a string is split into numeric and non-numeric sections.
//! The comparison starts at the first group and if no group is (by any of the rules) larger than the other
//! the comparison moves on to the next section. For comparison of sections the following rules are
//! used.
//!
//! * Any non-numbers are compared using their usual compare methods
//! * Numbers are always greater than nun-numbers
//! * Numeric sequences are ordered by their numeric value
//! * Empty sequences are always smaller than non-empty ones
//!
//!
//! These examples should give you some idea of how this works out in practice:
//!
//! ```
//! use humanesort::HumaneSortable;
//! let mut a = ["lol-1", "lal-2"];
//! a.humane_sort();
//! assert_eq!(a, ["lal-2", "lol-1"])
//! ```
//!
//! ```
//! use humanesort::HumaneSortable;
//! let mut a = ["13-zzzz", "1-ffff", "12-aaaa"];
//! a.humane_sort();
//! assert_eq!(a, ["1-ffff", "12-aaaa", "13-zzzz"])
//! ```
extern crate unicode_segmentation;
pub mod prelude;
use std::iter::Peekable;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    #[test]
    fn sorting_test() {
        use ::SortingType;
        let s = "11LOL";
        let fun = &|x: &str| -> SortingType {
            if x.chars().all(|c| char::is_numeric(c)) {
                return SortingType::Numeric
            } else {
                return SortingType::NonNumeric
            }
        };
        let mut it = ::TokenIterator::new(s, fun);
        assert_eq!(it.next().unwrap().0, "11");
        assert_eq!(it.next().unwrap().0, "LOL");
    }

    #[test]
    fn sort_slice() {
        use HumaneSortable;
        let mut strings = vec!["11", "2", "a", "1"];
        strings.humane_sort();
        assert_eq!(vec!["1", "2", "11", "a"], strings);
        let mut sort_me = vec!["something-11", "something-1", "something-2"];
        sort_me.humane_sort();
        assert_eq!(vec!["something-1", "something-2", "something-11"], sort_me);
    }
}

fn sorting_type(x: &str) -> SortingType {
    let num: Result<u64, _> = x.parse();
    match num {
        Ok(_) => SortingType::Numeric,
        _ => SortingType::NonNumeric
    }
}

/// Trait for collections of `HumaneOrder` types.
pub trait HumaneSortable {
    fn humane_sort(&mut self);
}

impl<T> HumaneSortable for [T] where T: HumaneOrder {
    fn humane_sort(&mut self) {
        self.sort_by(|a, b| a.humane_cmp(b))
    }
}

/// Trait for types that can be ordered in a human friendly way.
pub trait HumaneOrder {
    fn humane_cmp(&self, other: &Self) -> Ordering;
}

impl<T> HumaneOrder for T where T: AsRef<str> {
    fn humane_cmp(&self, other: &Self) -> Ordering {
        let sorting_type_function = &sorting_type;
        let mut self_tokens = TokenIterator::new(self.as_ref(), sorting_type_function);
        let mut other_tokens = TokenIterator::new(other.as_ref(), sorting_type_function);
        loop {
            match (self_tokens.next(), other_tokens.next()) {
                (None, None) => return Ordering::Equal,
                (None, _) => return Ordering::Less,
                (_, None) => return Ordering::Greater,
                (Some(ours), Some(theirs)) => {
                    match (ours.1, theirs.1) {
                        (SortingType::Numeric, SortingType::NonNumeric) => return Ordering::Less,
                        (SortingType::NonNumeric, SortingType::Numeric) => return Ordering::Greater,
                        (SortingType::Numeric, SortingType::Numeric) => {
                            let cmp = ours.0.parse::<usize>().unwrap().cmp(&theirs.0.parse::<usize>().unwrap());
                            if cmp != Ordering::Equal {
                                return cmp
                            }
                        }
                        (SortingType::NonNumeric, SortingType::NonNumeric) => {
                            let cmp = ours.0.cmp(theirs.0);
                            if cmp != Ordering::Equal {
                                return cmp
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum SortingType {
    Numeric,
    NonNumeric
}

struct TokenIterator<'a, T> where T: Eq + Copy + 'a {
    token_type: &'a Fn(&str) -> T,
    string: &'a str,
    grapheme_iterator: Peekable<GraphemeIndices<'a>>
}

impl<'a, T> TokenIterator<'a, T> where T: Eq + Copy {
    fn new(s: &'a str, func: &'a Fn(&str) -> T) -> Self {
        TokenIterator {
            token_type: func,
            string: s,
            grapheme_iterator: UnicodeSegmentation::grapheme_indices(&s[..], true).peekable()
        }
    }
}

impl<'a, T> Iterator for TokenIterator<'a, T> where T: Eq + Copy {
    type Item = (&'a str, T);

    fn next(&mut self) -> Option<(&'a str, T)> {
        let (first_index, mut grapheme) = match self.grapheme_iterator.next() {
            Some((i, s)) => (i, s),
            None => return None // This is only reached when the first element is None
        };
        loop {
            let current_type = (self.token_type)(grapheme);
            let (next_index, next_grapheme) = match self.grapheme_iterator.peek() {
                Some(&(i, g)) => (i, g),
                None => return Some((&self.string[first_index..self.string.len()], (self.token_type)(grapheme)))
            };
            if current_type != (self.token_type)(next_grapheme) {
                return Some((&self.string[first_index..next_index], current_type))
            }
            let tup = match self.grapheme_iterator.next() {
                Some((i, s)) => (i, s),
                None => return None // This is only reached when the first element is None
            };
            grapheme = tup.1;
        }
    }
}
