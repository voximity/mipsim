use std::str::CharIndices;

pub struct IndexedChars<'a> {
    slice: &'a str,
    iter: CharIndices<'a>,
    peeked: Option<Option<(usize, char)>>,
    offset: usize,
}

impl<'a> Iterator for IndexedChars<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take().unwrap_or_else(|| self.iter.next()) {
            Some((offset, c)) => {
                self.offset = self.peek_boundary();
                Some((offset, c))
            }
            None => {
                self.offset = self.slice.len();
                None
            }
        }
    }
}

impl<'a> IndexedChars<'a> {
    pub fn new(slice: &'a str) -> Self {
        Self {
            slice,
            iter: slice.char_indices(),
            peeked: None,
            offset: 0,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn peek(&mut self) -> Option<&(usize, char)> {
        self.peeked.get_or_insert_with(|| self.iter.next()).as_ref()
    }

    pub fn peek_boundary(&mut self) -> usize {
        self.peek().map(|(idx, _)| *idx).unwrap_or(self.slice.len())
    }
}
