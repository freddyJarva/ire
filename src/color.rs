use std::fmt::{self, Display};

use colored::Colorize;
use regex::{Captures, Regex};
use std::ops::Deref;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
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
        let hs = vec![Color::Yellow, Color::Blue, Color::Red];
        let mut highlight_styles = hs.iter().cycle();

        let spans: Vec<Span> = self
            .iter()
            .map(|color_style| match color_style {
                ColorStyle::Normal(s) => Span::raw(s),
                ColorStyle::Highlight(s) => {
                    let style = match highlight_styles.next().unwrap() {
                        Color::Red => Style::default().fg(Color::Red),
                        Color::Yellow => Style::default().fg(Color::Yellow),
                        Color::Blue => Style::default().fg(Color::Blue),
                        _ => Style::default().fg(Color::Green),
                    };
                    // let span_style = Style::default().fg(Color::Yellow);

                    Span::styled(s, style)
                }
            })
            .collect();
        Spans::from(spans)
    }
}

pub fn filter_matches(contents: &Vec<String>, re: &Regex) -> Vec<String> {
    contents
        .iter()
        .filter(|s| re.is_match(s))
        .map(|s| s.to_string())
        .collect()
}

pub fn collect_matches(contents: &Vec<String>, re: &Regex) -> Vec<Vec<ColorStyle>> {
    let result: Vec<Vec<ColorStyle>> = contents
        .iter()
        .filter(|s| re.is_match(s))
        .map(|s| split_on_matches(s, &re.captures(s).unwrap()))
        .collect();
    result
}

fn split_on_matches(full_text: &str, captures: &regex::Captures) -> Vec<ColorStyle> {
    let mut result = Vec::new();

    match captures.len() {
        0..=1 => result.push(ColorStyle::Normal(full_text.to_string())),
        _ => {
            let mut previous_end = 0;
            for i in 1..captures.len() {
                if let Some(mat) = captures.get(i) {
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
            if previous_end != full_text.len() {
                result.push(ColorStyle::Normal(full_text[previous_end..].to_string()))
            }
        }
    }
    result
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    macro_rules! colorstyle {
        ($style:ident $string:expr) => {
            ColorStyle::$style($string.to_string())
        };
    }

    macro_rules! test_split_on_matches {
        ($($func_name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $func_name() {
                // Given
                let (re, content, expected) = $value;
                let re = Regex::new(re).unwrap();
                let captures = re.captures(content).unwrap();
                // When
                let actual: Vec<ColorStyle> = split_on_matches(content, &captures);

                // Then
                assert_eq!(expected, actual)
            }
        )*
        };
    }

    test_split_on_matches! {
        capture_split_matches_return_wrappers : (r".+(hello).+(world)", "lala hello bleble world", vec![
            colorstyle!(Normal "lala "),
            colorstyle!(Highlight "hello"),
            colorstyle!(Normal " bleble "),
            colorstyle!(Highlight "world"),
        ]),
        givenNoCaptureGroups_thenFullTextAsSingleElement : (r".*", "lala hello ", vec![colorstyle!(Normal "lala hello ")]),
        givenEmptyPattern_thenReturnFullTextAsSingleElement : (r"", "lala ", vec![colorstyle!(Normal "lala ")]),
        givenPartialMatch_thenReturnFullTextInElements : (r".*(lala)", "1337 lala hey ho!", vec![
            colorstyle!(Normal "1337 "),
            colorstyle!(Highlight "lala"),
            colorstyle!(Normal " hey ho!"),
        ]),
        givenNonCapturingGroup_thenUseNormalColorStyle : (r"(?:lala )(bleble)", "lala bleble", vec![
            colorstyle!(Normal "lala "),
            colorstyle!(Highlight "bleble"),
        ]),
        given0or1MatchReturnsNone_thenDoNotReturnIt : (r"(lala)?(bleble)", "bleble", vec![
            colorstyle!(Highlight "bleble"),
        ]),
        // given0toNMatchReturnsMultiple_thenReturnEachGroupAsSeparateHighlight : (r"(lala )*", "lala lala ", vec![
        //     colorstyle!(Highlight "lala "),
        //     colorstyle!(Highlight "lala "),
        // ]),

    }

    #[test]
    fn format_vec_colorstyle() {
        // Given
        let re = Regex::new(r".+(hello).+(world)").unwrap();
        let content = "lala hello bleble world";
        let captures = re.captures(content).unwrap();
        // When
        let actual = split_on_matches(content, &captures).highlight();

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

    macro_rules! svec {
        ($multiline_string:expr) => {
            $multiline_string
                .to_string()
                .split('\n')
                .map(|s| s.to_string())
                .collect()
        };
    }

    #[test]
    fn test_pattern_matching_list() {
        // Given
        let contents = svec!(
            "\
hello world
hello blabla world
"
        );
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
        let contents = vec![colorstyle!(Normal "lala "), colorstyle!(Highlight "hello")];
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

    #[test]
    fn givenMultipleHighLights_whenStyled_thenReturnSpansOfDifferentColors() {
        // Given
        let contents = vec![
            colorstyle!(Normal "lala "),
            colorstyle!(Highlight "hello"),
            colorstyle!(Highlight "blue"),
            colorstyle!(Highlight "red"),
            colorstyle!(Normal "world"),
        ];
        let yellow = Style::default().fg(Color::Yellow);
        let blue = Style::default().fg(Color::Blue);
        let red = Style::default().fg(Color::Red);
        let expected = Spans::from(vec![
            Span::raw("lala "),
            Span::styled("hello", yellow),
            Span::styled("blue", blue),
            Span::styled("red", red),
            Span::raw("world"),
        ]);
        // When
        let actual = contents.style();
        // Then
        assert_eq!(expected, actual)
    }
}
