//! A crate for sorting the way humans would.
//! 
//! This crate aims to provide the sorting behavior a human might expect.
//! Say you have a directory of files all called "Something-" with a sequential number appended.
//! With traditional sorting by character the file "Something-11" would occur after the file
//! "Something-2".
//! Often this is not the desired behavior, this crate implements a more human compatible ordering
//! by treating each occurrence of consecutive digits as a combined number in sorting.
//! 
//! The crate implements the type `HumaneOrder` for common types and `HumaneSortable` for slices of
//! `HumanOrder` types.
extern crate unicode_segmentation;
use std::iter::Peekable;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    #[test]
    fn sorting_test() {
        use ::SortingType;
        let s = "11LOL";
        let mut it = ::TokenIterator::new(s, Box::new(|x: &str| -> SortingType {
            if x.chars().all(|c| char::is_numeric(c)) {
                return SortingType::Numeric
            } else {
                return SortingType::NonNumeric
            }
        }));
        assert_eq!(it.next().unwrap().0, "11");
        assert_eq!(it.next().unwrap().0, "LOL");
    }

    #[test]
    fn sort_slice() {
        use HumaneSortable;
        let mut strings = vec!["11", "2", "a", "1"];
        strings.humane_sort();
        assert_eq!(vec!["1", "2", "11", "a"], strings);
    }
}

fn sorting_type(x: &str) -> SortingType {
    let num: Result<u64, _> = x.parse();
    match num {
        Ok(_) => SortingType::Numeric,
        _ => SortingType::NonNumeric
    }
}

/// A type that can be sorted in a more human compatible fashion should implement this.
pub trait HumaneSortable {
    fn humane_sort(&mut self);
}

impl<T> HumaneSortable for [T] where T: HumaneOrder {
    fn humane_sort(&mut self) {
        self.sort_by(|a, b| a.humane_cmp(b))
    }
}

pub trait HumaneOrder {
    fn humane_cmp(&self, other: &Self) -> Ordering;
}

impl<T> HumaneOrder for T where T: AsRef<str> {
    fn humane_cmp(&self, other: &Self) -> Ordering {
        let mut self_tokens = TokenIterator::new(self.as_ref(), Box::new(sorting_type));
        let mut other_tokens = TokenIterator::new(other.as_ref(), Box::new(sorting_type));
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

#[derive(PartialEq, Eq, Debug)]
enum SortingType {
    Numeric,
    NonNumeric
}

struct TokenIterator<'a, T> where T: Eq { token_type: Box<Fn(&str) -> T>, string: &'a str,
    grapheme_iterator: Peekable<GraphemeIndices<'a>>
}

impl<'a, T> TokenIterator<'a, T> where T: Eq {
    fn new(s: &'a str, func: Box<Fn(&str) -> T>) -> Self {
        TokenIterator {
            token_type: func,
            string: s,
            grapheme_iterator: UnicodeSegmentation::grapheme_indices(&s[..], true).peekable()
        }
    }
}

impl<'a, T> Iterator for TokenIterator<'a, T> where T: Eq {
    type Item = (&'a str, T);

    fn next(&mut self) -> Option<(&'a str, T)> {
        let (first_index, mut grapheme) = match self.grapheme_iterator.next() {
            Some((i, s)) => (i, s),
            None => return None // This is only reached when the first element is None
        };
        let mut index = first_index;
        loop {
            let current_type = (self.token_type)(grapheme);
            let next_grapheme = match self.grapheme_iterator.peek() {
                Some(&(_, t)) => t,
                None => {return Some((&self.string[first_index..index+1], (self.token_type)(grapheme)))}
            };
            if current_type != (self.token_type)(next_grapheme) {
                return Some((&self.string[first_index..index+1], current_type))
            }
            let tup = match self.grapheme_iterator.next() {
                Some((i, s)) => (i, s),
                None => return None // This is only reached when the first element is None
            };
            index = tup.0;
            grapheme = tup.1;
        }
    }
}
