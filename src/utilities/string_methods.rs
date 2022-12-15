pub trait SMISString {
    fn count_words(&self) -> usize;
    fn get_word(&self, index: usize) -> Option<&str>;
    fn without_first_word(&self) -> String;
}

impl<'a> SMISString for &'a str {
    fn count_words(&self) -> usize {
        self.split_whitespace().count()
    }

    fn get_word(&self, index: usize) -> Option<&str> {
        // Split the string into words and pick the word at the given index
        self.split_whitespace().nth(index)
    }

    fn without_first_word(&self) -> String {
        // Split the string into words and collect the words into a vector
        let words: Vec<&str> = self.split_whitespace().collect();

        // Join the words into a string, starting from the second word
        words[1..].join(" ")
    }
}
