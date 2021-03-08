use regex::{Captures, Regex};

#[derive(Debug, PartialEq)]
pub enum MatchType {
    Normal(String),
    Group(String),
}

#[derive(Debug, PartialEq)]
pub struct MatchSet {
    items: Vec<MatchType>,
    full_text: String,
}

pub fn into_matchset(full_text: &str, captures: &regex::Captures) -> MatchSet {
    let mut items = Vec::new();

    match captures.len() {
        0..=1 => items.push(MatchType::Normal(full_text.to_string())),
        _ => {
            let mut previous_end = 0;
            for i in 1..captures.len() {
                if let Some(mat) = captures.get(i) {
                    if mat.start() != previous_end {
                        items.push(MatchType::Normal(
                            full_text[previous_end..mat.start()].to_string(),
                        ));
                    }
                    items.push(MatchType::Group(
                        full_text[mat.start()..mat.end()].to_string(),
                    ));
                    previous_end = mat.end();
                }
            }
            if previous_end != full_text.len() {
                items.push(MatchType::Normal(full_text[previous_end..].to_string()))
            }
        }
    }
    MatchSet {
        full_text: full_text.to_string(),
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! matchtype {
        ($style:ident $string:expr) => {
            MatchType::$style($string.to_string())
        };
    }

    #[test]
    fn filter_matches_returns_iterator() {
        let re = Regex::new(r"(hello) world").unwrap();
        let expected = MatchSet {
            items: vec![matchtype!(Group "hello"), matchtype!(Normal " world")],
            full_text: "hello world".to_string(),
        };
        let captures = re.captures("hello world").unwrap();
        // When
        let actual = into_matchset("hello world", &captures);
        // Then
        assert_eq!(expected, actual)
    }
}
