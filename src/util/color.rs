use colored::Colorize;
use regex::{Captures, Regex};
pub trait Colorized {
    fn highlight(&self) -> String;
}

impl Colorized for Captures<'_> {
    fn highlight(&self) -> String {
        let mut result = self.get(0).unwrap().as_str().to_string();
        match &self.len() {
            1 => {}
            _ => {
                for i in 1..self.len() {
                    result = result.replacen(
                        self.get(i).unwrap().as_str(),
                        &format!("#{}#", self.get(i).unwrap().as_str()),
                        1,
                    );
                }
            }
        }
        result
    }
}

fn split_on_matches(captures: &regex::Captures) -> Vec<String> {
    let mut result = Vec::new();
    let full_mat = captures.get(0).unwrap();
    let full_text = String::from(full_mat.as_str());

    let mut previous_end = 0;
    for i in 1..captures.len() {
        let mat = captures.get(i).unwrap();
        if mat.start() != previous_end {
            result.push(full_text[previous_end..mat.start()].to_string());
        }
        result.push(full_text[mat.start()..mat.end()].to_string());
        previous_end = mat.end();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_group_match() {
        // Given
        let re = Regex::new(r"(hello) (world!)").unwrap();
        let captures = re.captures("hello world!").unwrap();
        // When
        let actual = captures.highlight();
        // Then
        assert_eq!("#hello# #world!#", actual);
    }

    #[test]
    fn capture_split_matches() {
        // Given
        let re = Regex::new(r".+(hello).+(world)").unwrap();
        let captures = re.captures("lala hello bleble world").unwrap();
        // When
        let actual: Vec<String> = split_on_matches(&captures);

        // Then
        assert_eq!(vec!["lala ", "hello", " bleble ", "world"], actual)
    }
}
