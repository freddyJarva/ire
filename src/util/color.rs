use std::fmt::{self, Display};

use colored::Colorize;
use regex::{Captures, Regex};
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::ListItem,
};
pub trait Colorized {
    fn highlight(&self) -> String;
}

pub trait Styled {
    fn style(&self) -> Spans;
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

impl Styled for Vec<ColorStyle> {
    fn style(&self) -> Spans {
        let spans: Vec<Span> = self
            .iter()
            .map(|color_style| match color_style {
                ColorStyle::Normal(s) => Span::raw(s),
                ColorStyle::Highlight(s) => {
                    let span_style = Style::default().fg(Color::Yellow);
                    Span::styled(s, span_style)
                }
            })
            .collect();
        Spans::from(spans)
    }
}

pub fn collect_matches(contents: &String, re: &Regex) -> Vec<Vec<ColorStyle>> {
    // let pattern_matches: Vec<ListItem> = contents
    let mats: Vec<&str> = contents.split('\n').filter(|s| re.is_match(s)).collect();
    let result: Vec<Vec<ColorStyle>> = mats
        .iter()
        .map(|s| split_on_matches(&re.captures(s).unwrap()))
        .collect();
    println!("Matches after processing: {:?}", result);
    result
}

fn split_on_matches(captures: &regex::Captures) -> Vec<ColorStyle> {
    let mut result = Vec::new();
    let full_mat = captures.get(0).unwrap();
    let full_text = String::from(full_mat.as_str());

    let mut previous_end = 0;
    if let 1 = captures.len() {
        result.push(ColorStyle::Normal(captures[0].to_string()))
    } else {
        {
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
        }
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
#[allow(non_snake_case)]
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
    fn givenNoCaptureGroups_whenSplitOnMatches_thenReturnVectorWithColorStyleNormalElement() {
        // Given
        let re = Regex::new(r".*").unwrap();
        let captures = re.captures("lala hello bleble world").unwrap();
        // When
        let actual: Vec<ColorStyle> = split_on_matches(&captures);
        // Then
        assert_eq!(
            vec![ColorStyle::Normal("lala hello bleble world".to_string())],
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

    #[test]
    fn test_pattern_matching_list() {
        // Given
        let contents = "\
hello world
hello blabla world
"
        .to_string();
        let re = Regex::new(r"(hello).+(world)").unwrap();
        // When
        let actual: Vec<String> = collect_matches(&contents, &re)
            .iter()
            .map(|v| v.highlight())
            .collect();
        assert_eq!(
            vec![
                format!("{} {}", "hello".red(), "world".red()),
                format!("{} blabla {}", "hello".red(), "world".red()),
            ],
            actual
        );
    }

    #[test]
    fn givenVecColorStyle_whenStyled_thenReturnSpans() {
        // Given
        let contents = vec![
            ColorStyle::Normal("lala ".to_string()),
            ColorStyle::Highlight("hello".to_string()),
        ];
        let expected_style = Style::default().fg(Color::Yellow);
        let expected = Spans::from(vec![
            Span::raw("lala "),
            Span::styled("hello", expected_style),
        ]);
        // When
        let actual = contents.style();
        // Then
        assert_eq!(expected, actual)
    }
}
