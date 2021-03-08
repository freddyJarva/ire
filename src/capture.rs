use regex::{Captures, Regex};

#[derive(Debug, PartialEq)]
pub enum MatchType {
    Normal(String),
    Group(String),
}

#[derive(Debug, PartialEq)]
pub struct MatchSet {
    pub items: Vec<MatchType>,
    pub full_text: String,
}

impl MatchSet {
    pub fn to_csv_row(&self) -> String {
        self.to_strings().join(",")
    }

    fn to_strings(&self) -> Vec<String> {
        let res: Vec<String> = self
            .items
            .iter()
            .filter(|mt| match mt {
                MatchType::Group(_) => true,
                _ => false,
            })
            .map(|mt| match mt {
                MatchType::Group(s) => s.to_string(),
                _ => "".to_string(),
            })
            .collect();
        res
    }

    pub fn to_tsv_row(&self) -> String {
        self.to_strings().join("\t")
    }
}

impl Default for MatchSet {
    fn default() -> Self {
        MatchSet {
            items: Vec::new(),
            full_text: "".to_string(),
        }
    }
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

pub fn filter_matches<'a>(contents: &'a [String], re: &Regex) -> Vec<(&'a str, Captures<'a>)> {
    contents
        .iter()
        .map(String::as_str)
        .filter(|s| re.is_match(s))
        .map(|s| (s, re.captures(s).unwrap()))
        .collect()
}

pub fn into_matchsets(captures: &[(&str, Captures)]) -> Vec<MatchSet> {
    let result: Vec<MatchSet> = captures
        .iter()
        .map(|(s, cap)| into_matchset(s, cap))
        .collect();
    result
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
        // TODO given0toNMatchReturnsMultiple_thenReturnEachPartAsSeparateGroup : (r"(lala )*", "lala lala ", vec![
        //     matchtype!(Group "lala "),
        //     matchtype!(Group "lala "),
        // ]),
    }

    macro_rules! test_print_options {
        ($($func_name:ident: $test_name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $test_name() {
                    // Given
                    let (expected, items) = $values;
                    let mut match_set = MatchSet::default();
                    match_set.items = items;
                    assert_eq!(expected, &match_set.$func_name())
                }
            )*
        }
    }

    test_print_options! {
        to_csv_row : return_comma_separated_row :  ("remain,remain also", vec![
            matchtype!(Normal "drop"),
            matchtype!(Group "remain"),
            matchtype!(Group "remain also"),
        ]),
        to_tsv_row : return_tab_separated_row : ("remain\tremain also", vec![
            matchtype!(Normal "drop"),
            matchtype!(Group "remain"),
            matchtype!(Group "remain also"),
        ]),
    }
}
