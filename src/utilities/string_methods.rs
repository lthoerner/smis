pub trait SMISString {
    // fn trim_one_start_matches(self, c: char);
}

impl<'a> SMISString for &'a str {
    // fn trim_one_start_matches(self, c: char) {
    //     // Ensure the string starts with the given char
    //     if !self.starts_with(c) { return; }

    //     // Remove the first character from the string
    //     let (_, self) = self.split_at(1);
    // }
}