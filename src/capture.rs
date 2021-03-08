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
#[allow(non_snake_case)]
mod tests {
    use super::*;

    macro_rules! matchtype {
        ($style:ident $string:expr) => {
            MatchType::$style($string.to_string())
        };
    }

    macro_rules! test_into_matchset {
        ($($func_name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $func_name() {
                // Given
                let (re, content, items) = $value;
                let re = Regex::new(re).unwrap();
                let captures = re.captures(content).unwrap();
                let expected = MatchSet {
                    full_text: content.to_string(),
                    items: items
                };
                // When
                let actual: MatchSet = into_matchset(content, &captures);

                // Then
                assert_eq!(expected, actual)
            }
        )*
        };
    }

    test_into_matchset! {
        into_match_set_basetest : (r".+(hello).+(world)", "lala hello bleble world", vec![
            matchtype!(Normal "lala "),
            matchtype!(Group "hello"),
            matchtype!(Normal " bleble "),
            matchtype!(Group "world"),
        ]),
        givenNoCaptureGroups_thenFullTextAsSingleElement : (r".*", "lala hello ", vec![matchtype!(Normal "lala hello ")]),
        givenEmptyPattern_thenReturnFullTextAsSingleElement : (r"", "lala ", vec![matchtype!(Normal "lala ")]),
        givenPartialMatch_thenReturnRemainingSubstringsAsNormal : (r".*(lala)", "1337 lala hey ho!", vec![
            matchtype!(Normal "1337 "),
            matchtype!(Group "lala"),
            matchtype!(Normal " hey ho!"),
        ]),
        givenNonCapturingGroup_thenReturnNormal : (r"(?:lala )(bleble)", "lala bleble", vec![
            matchtype!(Normal "lala "),
            matchtype!(Group "bleble"),
        ]),
        given0or1MatchReturnsNone_thenDoNotReturnIt : (r"(lala)?(bleble)", "bleble", vec![
            matchtype!(Group "bleble"),
        ]),
        given0toNMatchReturnsMultiple_thenReturnEachPartAsSeparateGroup : (r"(lala )*", "lala lala ", vec![
            matchtype!(Group "lala "),
            matchtype!(Group "lala "),
        ]),

    }
}
