use std::{iter::Peekable, str::Chars};

pub struct ShellParser {
    result: Vec<String>,
    string_to_push: String,
    in_single_quotes: bool,
    in_double_quotes: bool,
    found_space: bool,
    in_backslash: bool,
}

impl ShellParser {
    pub fn parsed_input(coming_input: &String) -> Vec<String> {
        let mut iterable_char: Peekable<Chars<'_>>;
        let mut parser_collection: ShellParser;

        parser_collection = ShellParser {
            result: Vec::new(),
            string_to_push: String::new(),
            in_single_quotes: false,
            in_double_quotes: false,
            found_space: false,
            in_backslash: false,
        };
        iterable_char = coming_input.chars().peekable();
        while let Some(&char_to_analyze) = iterable_char.peek() {
            parser_collection.populate_result(char_to_analyze);
            iterable_char.next();
        }
        if !parser_collection.string_to_push.is_empty() {
            parser_collection
                .result
                .push(parser_collection.string_to_push.clone());
            parser_collection.string_to_push.clear();
        }
        parser_collection.result
    }

    fn populate_result(&mut self, char_to_analyze: char) {
        match char_to_analyze {
            '"' => {
                if self.found_space {
                    self.found_space = false;
                }
                if self.in_single_quotes {
                    self.string_to_push.push(char_to_analyze);
                    return;
                } else {
                    if !self.in_backslash {
                        self.in_double_quotes = !self.in_double_quotes;
                    } else {
                        self.string_to_push.push(char_to_analyze);
                        self.in_backslash = false;
                        return;
                    }
                }
            }
            '\\' => {
                if self.found_space {
                    self.found_space = false;
                }
                if self.in_single_quotes {
                    self.string_to_push.push(char_to_analyze);
                    return;
                }
                self.in_backslash = !self.in_backslash;
                if !self.in_backslash {
                    self.string_to_push.push('\\');
                }
            }
            '\'' => {
                if self.found_space {
                    self.found_space = false;
                }
                if self.in_double_quotes {
                    self.string_to_push.push(char_to_analyze);
                    return;
                } else {
                    self.in_single_quotes = !self.in_single_quotes;
                    return;
                }
            }
            ' ' => {
                if self.in_single_quotes || self.in_double_quotes || self.in_backslash {
                    self.string_to_push.push(char_to_analyze);
                    if self.in_backslash {
                        self.in_backslash = !self.in_backslash;
                    }
                    return;
                }
                if !self.found_space {
                    self.result.push(self.string_to_push.clone());
                    self.string_to_push.clear();
                    self.found_space = true;
                }
            }
            _ => {
                if self.found_space {
                    self.found_space = false;
                }
                if self.in_double_quotes && self.in_backslash {
                    self.string_to_push.push('\\');
                    self.in_backslash = false;
                }
                self.string_to_push.push(char_to_analyze);
            }
        };
    }
}
