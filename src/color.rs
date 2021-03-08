use std::fmt::{self, Display};

use colored::Colorize;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::capture::{MatchSet, MatchType};
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

impl Styled for MatchSet {
    fn style(&self) -> Spans {
        let hs = vec![Color::Yellow, Color::Blue, Color::Red];
        let mut highlight_styles = hs.iter().cycle();

        let spans: Vec<Span> = self
            .items
            .iter()
            .map(|color_style| match color_style {
                MatchType::Normal(s) => Span::raw(s),
                MatchType::Group(s) => {
                    let style = match highlight_styles.next().unwrap() {
                        Color::Red => Style::default().fg(Color::Red),
                        Color::Yellow => Style::default().fg(Color::Yellow),
                        Color::Blue => Style::default().fg(Color::Blue),
                        _ => Style::default().fg(Color::Green),
                    };
                    Span::styled(s, style)
                }
            })
            .collect();
        Spans::from(spans)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use regex::Regex;

    use super::*;

    macro_rules! colorstyle {
        ($style:ident $string:expr) => {
            ColorStyle::$style($string.to_string())
        };
    }

    #[test]
    fn display_colorstyle() {
        assert_eq!(
            "lala".red().to_string(),
            ColorStyle::Highlight("lala".to_string()).to_string()
        )
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
