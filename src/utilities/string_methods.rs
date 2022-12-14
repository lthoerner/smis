pub trait SMISString {
    fn count_words(&self) -> usize;
    fn get_word(&self, index: usize) -> &str;
}

impl<'a> SMISString for &'a str {
    fn count_words(&self) -> usize {
        self.split_whitespace().count()
    }

    fn get_word(&self, index: usize) -> &str {
        // TODO: Add some actual error handling here, or use Option
        if index > self.count_words() { panic!("Word {} out of bounds for string {} (size: {})", index, self, self.count_words()); }
        
        // Split the string into words and pick the word at the given index
        self.split_whitespace().nth(index).unwrap()
    }
}