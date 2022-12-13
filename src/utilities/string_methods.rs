pub trait SMISString {
    fn trim_one_start_matches(self, c: char);
}

pub impl<'a> SMISString for &'a str {
    fn trim_one_start_matches(self, c: char) {
        println!("worked!");
    }
}