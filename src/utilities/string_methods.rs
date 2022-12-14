pub trait SMISString {
    fn count_words(&self) -> usize;
    fn get_word(&self, index: usize) -> Option<&str>;
}

impl<'a> SMISString for &'a str {
    fn count_words(&self) -> usize {
        self.split_whitespace().count()
    }

    fn get_word(&self, index: usize) -> Option<&str> {
        // Split the string into words and pick the word at the given index
        self.split_whitespace().nth(index)
    }
}