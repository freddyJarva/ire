use regex::{Captures, Match, Regex};

#[derive(Debug, PartialEq)]
pub enum MatchType {
    Normal(String),
    Group(String),
}

#[derive(Debug, PartialEq)]
pub struct MatchItem<'a> {
    pub text: &'a str,
    pub mtype: MatchType,
}

#[derive(Debug)]
pub struct MatchSet<'a> {
    pub full_text: &'a str,
    pub re: &'a Regex,
    // pub items: Vec<MatchType>,
}

impl<'a> MatchSet<'a> {
    pub fn from(full_text: &'a str, re: &'a Regex) -> Self {
        MatchSet { full_text, re }
    }

    pub fn raw_line(&self) -> String {
        self.full_text.to_string()
    }

    pub fn to_csv_row(&self) -> String {
        self.to_strings().join(",")
    }

    pub fn to_strings(&self) -> Vec<String> {
        let mut items = Vec::new();
        let captures = self.re.captures(self.full_text).unwrap();

        match captures.len() {
            0..=1 => items.push(MatchType::Normal(self.full_text.to_string())),
            _ => {
                let mut previous_end = 0;
                for i in 1..captures.len() {
                    if let Some(mat) = captures.get(i) {
                        if mat.start() != previous_end {
                            items.push(MatchType::Normal(
                                self.full_text[previous_end..mat.start()].to_string(),
                            ));
                        }
                        items.push(MatchType::Group(
                            self.full_text[mat.start()..mat.end()].to_string(),
                        ));
                        previous_end = mat.end();
                    }
                }
                if previous_end != self.full_text.len() {
                    items.push(MatchType::Normal(
                        self.full_text[previous_end..].to_string(),
                    ))
                }
            }
        }
        let res: Vec<String> = items
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

    pub fn to_matchtypes(&self) -> Vec<MatchType> {
        let mut items = Vec::new();
        let captures = self.re.captures(self.full_text).unwrap();

        match captures.len() {
            0..=1 => items.push(MatchType::Normal(self.full_text.to_string())),
            _ => {
                let mut previous_end = 0;
                for i in 1..captures.len() {
                    if let Some(mat) = captures.get(i) {
                        if mat.start() != previous_end {
                            items.push(MatchType::Normal(
                                self.full_text[previous_end..mat.start()].to_string(),
                            ));
                        }
                        items.push(MatchType::Group(
                            self.full_text[mat.start()..mat.end()].to_string(),
                        ));
                        previous_end = mat.end();
                    }
                }
                if previous_end != self.full_text.len() {
                    items.push(MatchType::Normal(
                        self.full_text[previous_end..].to_string(),
                    ))
                }
            }
        }
        items
    }

    pub fn to_tsv_row(&self) -> String {
        self.to_strings().join("\t")
    }
}

pub fn filter_matches<'a>(contents: &'a [String], re: &Regex) -> Vec<&'a str> {
    contents
        .iter()
        .map(String::as_str)
        .filter(|s| re.is_match(s))
        .collect()
}

pub fn into_matchsets<'a>(text_lines: &[&'a str], re: &'a Regex) -> Vec<MatchSet<'a>> {
    let result: Vec<MatchSet> = text_lines.iter().map(|s| MatchSet::from(&s, &re)).collect();
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

    macro_rules! test_matchset_from {
        ($($func_name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $func_name() {
                // Given
                let (re, content, items) = $value;
                let re = Regex::new(re).unwrap();
                // let expected = MatchSet {
                //     full_text: content,
                //     re: &re
                // };
                // When
                let actual: Vec<MatchType> = MatchSet::from(content, &re).to_matchtypes();

                // Then
                assert_eq!(items, actual)
            }
        )*
        };
    }

    test_matchset_from! {
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
    }

    macro_rules! test_print_options {
        ($($func_name:ident: $test_name:ident: $values:expr,)*) => {
            $(
                #[test]
                fn $test_name() {
                    // Given
                    let (expected, full_text, re) = $values;
                    let match_set = MatchSet{full_text, re: &Regex::new(re).unwrap()};
                    assert_eq!(expected, &match_set.$func_name())
                }
            )*
        }
    }

    test_print_options! {
        to_csv_row : return_comma_separated_row :  ("remain,remain also", "drop remain remain also", r"\w+ (\w+) (\w+ \w+)"),
        to_tsv_row : return_tab_separated_row : ("remain\tremain also", "drop remain remain also", r"\w+ (\w+) (\w+ \w+)"),
    }
}
