extern crate unicode_segmentation;
use std::iter::Peekable;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use ::SortingType;
        let s = "11LOL";
        let mut it = ::TokenIterator::new(s, Box::new(|x: &str| -> SortingType {
            let num: Result<u64, _> = x.parse();
            match num {
                Ok(_) => SortingType::Number,
                _ => SortingType::Str
            }
        }));
        assert_eq!(it.next().unwrap().0, "11");
        assert_eq!(it.next().unwrap().0, "LOL");
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum SortingType {
    Number,
    Str
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
        return None
    }
}
