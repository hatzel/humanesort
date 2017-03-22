extern crate unicode_segmentation;
use std::iter::Peekable;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use ::SortingType;
        let s = "11LOL";
        let mut it = ::TokenIterator::new(s, Box::new(|x: &str| -> SortingType {
            let num: Result<u64, _> = x.parse();
            match num {
                Ok(_) => SortingType::Numeric,
                _ => SortingType::NonNumeric
            }
        }));
        assert_eq!(it.next().unwrap().0, "11");
        assert_eq!(it.next().unwrap().0, "LOL");
    }
    
    #[test]
    fn sort() {
        use ::HumanString;
        let strings = vec!["11", "2", "a", "1"];
        let mut humans: Vec<HumanString> = strings.iter().map(|s| ::HumanString::new(s)).collect();
        humans.sort();
        let sorted_strings: Vec<String> = humans.into_iter().map(|hs| hs.data).collect();
        assert_eq!(vec!["1", "2", "11", "a"], sorted_strings);
    }
}

#[derive(PartialEq, Eq, Debug)]
struct HumanString {
    data: String
}

impl HumanString {
    fn new(s: &str) -> Self {
        HumanString {
            data: s.to_owned()
        }
    }
}

fn sorting_type(x: &str) -> SortingType {
    let num: Result<u64, _> = x.parse();
    match num {
        Ok(_) => SortingType::Numeric,
        _ => SortingType::NonNumeric
    }
}

impl Ord for HumanString {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut self_tokens = TokenIterator::new(&self.data, Box::new(sorting_type));
        let mut other_tokens = TokenIterator::new(&other.data, Box::new(sorting_type));
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

impl PartialOrd for HumanString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum SortingType {
    Numeric,
    NonNumeric
}

struct TokenIterator<'a, T> where T: Eq {
    token_type: Box<Fn(&str) -> T>,
    string: &'a str,
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

impl<'a, T> Iterator for TokenIterator<'a, T> where T: Eq + Clone {
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
