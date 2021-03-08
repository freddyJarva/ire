#[cfg(test)]
mod tests {
    use regex::Regex;
    #[test]
    fn match_objects_location_end_is_exclusive_() {
        let re = Regex::new(r"(\w+)+").unwrap();
        let captures = re.captures("hello world").unwrap();

        let actual = captures.get(1).unwrap();
        assert_eq!(0, actual.start());
        assert_eq!(5, actual.end());
    }

    #[test]
    fn expand_adds_replacement_to_end_of_dst() {
        let re = Regex::new(r"(\w+) (\w+)").unwrap();
        let captures = re.captures("hello world").unwrap();
        let mut dst = "what is this".to_string();
        captures.expand(" hihi $1 hoho $2", &mut dst);
        assert_eq!("what is this hihi hello hoho world", dst)
    }
}
