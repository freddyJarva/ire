use std::cmp::{max, min};

pub struct Input {
    pub text: String,
    pub mode: InputMode,
    idx: usize,
}

impl Input {
    pub fn idx(&self) -> &usize {
        &self.idx
    }
}

impl Default for Input {
    fn default() -> Input {
        Input {
            text: "".to_string(),
            mode: InputMode::Normal,
            idx: 0,
        }
    }
}

impl Editable for Input {
    fn left(&mut self) {
        match &self.idx {
            0 => {}
            1..=1000 => self.idx -= 1,
            _ => {}
        }
        ()
    }

    fn right(&mut self) {
        self.idx = min(self.text.len(), self.idx + 1)
    }

    fn delete(&mut self) {
        match &self.idx {
            0 => {}
            _ => {
                self.idx -= 1;
                self.text.remove(self.idx);
            }
        }
    }

    fn enter(&mut self) {
        todo!()
    }

    fn esc(&mut self) {
        todo!()
    }

    fn add(&mut self, c: char) {
        self.text.insert(self.idx, c);
        self.idx += 1;
    }
}

pub trait Editable {
    fn left(&mut self);
    fn right(&mut self);
    fn delete(&mut self);
    fn enter(&mut self);
    fn esc(&mut self);
    fn add(&mut self, c: char);
}

pub enum InputMode {
    Normal,
    Editing,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_move {
        ($($input_func:ident: $func_name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $func_name() {
                    // Given
                    let (text, idx, expected_idx) = $value;
                    let mut input = Input {text : text.to_string(), mode : InputMode::Normal, idx};
                    // When
                    input.$input_func();
                    // Then
                    assert_eq!(expected_idx, input.idx)
                }
            )*
        };
    }

    test_move! {
        left : when_on_0_then_remain : ("", 0, 0),
        left : decrement_idx_by_1: ("hello", 3, 2),
        right: when_idx_equals_textlength_then_remain : ("hello", 5, 5),
        right: increment_idx_by_1 : ("hello", 3, 4),
    }

    macro_rules! test_edit {
        ($($input_func:ident: $func_name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $func_name() {
                    // Given
                    let (text, idx, c, expected_idx, expected_text) = $value;
                    let mut input = Input {text : text.to_string(), mode : InputMode::Normal, idx};
                    // When
                    input.$input_func(c);
                    // Then
                    assert_eq!(expected_idx, input.idx);
                    assert_eq!(expected_text, input.text);
                }
            )*
        };
    }

    test_edit! {
        add : when_add_char_then_increment_idx_by_1 : ("bolloc", 6, 'k', 7, "bollock"),
        add : char_is_inserted_at_index : ("ollock", 0, 'b', 1, "bollock"),
    }

    #[test]
    fn delete_nothing_on_idx_0() {
        let mut input = Input {
            text: "bla".to_string(),
            mode: InputMode::Normal,
            idx: 0,
        };
        input.delete();
        assert_eq!("bla", &input.text);
    }

    #[test]
    fn delete_char_and_decrement_idx() {
        let mut input = Input {
            text: "bla".to_string(),
            mode: InputMode::Normal,
            idx: 2,
        };
        input.delete();
        assert_eq!(1, *input.idx());
        assert_eq!("ba", &input.text);
    }
}
