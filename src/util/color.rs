use std::fmt::{self, Display};

use colored::Colorize;
use regex::{Captures, Regex};
use tui::{
    text::{Span, Spans},
    widgets::ListItem,
};
pub trait Colorized {
    fn highlight(&self) -> String;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ColorStyle {
    Normal(String),
    Highlight(String),
}

impl Display for ColorStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ColorStyle::Normal(s) => write!(f, "{}", s),
            ColorStyle::Highlight(s) => write!(f, "{}", s.red()),
        }
    }
}

impl Colorized for Vec<ColorStyle> {
    fn highlight(&self) -> String {
        self.iter()
            .fold("".to_string(), |s, color| format!("{}{}", s, color))
    }
}

pub fn collect_matches(contents: &String, re: &Regex) -> Vec<Vec<ColorStyle>> {
    // let pattern_matches: Vec<ListItem> = contents
    let pattern_matches: Vec<Vec<ColorStyle>> = contents
        .split('\n')
        .filter(|s| re.is_match(s))
        .map(|s| split_on_matches(&re.captures(s).unwrap()))
        .collect();
    pattern_matches
}

fn split_on_matches(captures: &regex::Captures) -> Vec<ColorStyle> {
    let mut result = Vec::new();
    let full_mat = captures.get(0).unwrap();
    let full_text = String::from(full_mat.as_str());

    let mut previous_end = 0;
    for i in 1..captures.len() {
        let mat = captures.get(i).unwrap();
        if mat.start() != previous_end {
            result.push(ColorStyle::Normal(
                full_text[previous_end..mat.start()].to_string(),
            ));
        }
        result.push(ColorStyle::Highlight(
            full_text[mat.start()..mat.end()].to_string(),
        ));
        previous_end = mat.end();
    }
    result
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
    fn capture_split_matches_return_wrappers() {
        // Given
        let re = Regex::new(r".+(hello).+(world)").unwrap();
        let captures = re.captures("lala hello bleble world").unwrap();
        // When
        let actual: Vec<ColorStyle> = split_on_matches(&captures);

        // Then
        assert_eq!(
            vec![
                ColorStyle::Normal("lala ".to_string()),
                ColorStyle::Highlight("hello".to_string()),
                ColorStyle::Normal(" bleble ".to_string()),
                ColorStyle::Highlight("world".to_string()),
            ],
            actual
        )
    }

    #[test]
    fn format_vec_colorstyle() {
        // Given
        let re = Regex::new(r".+(hello).+(world)").unwrap();
        let captures = re.captures("lala hello bleble world").unwrap();
        // When
        let actual = split_on_matches(&captures).highlight();

        // Then
        assert_eq!(
            format!("lala {} bleble {}", "hello".red(), "world".red()),
            actual
        )
    }

    #[test]
    fn display_colorstyle() {
        assert_eq!(
            "lala".red().to_string(),
            ColorStyle::Highlight("lala".to_string()).to_string()
        )
    }
}
