use crate::parser::errors::LexerError;
use crate::parser::token::Token;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::FromStr;

const SEMICOLON: char = ';';
const DOUBLE_QUOTES: char = '"';
const SINGLE_QUOTES: char = '\'';
const UNDERSCORE: char = '_';
const BYTES_PREFIX: char = 'b';
const DOT_SEPERATOR: char = '.';

pub struct Lexer<T: Iterator<Item = char>> {
    input: Peekable<T>,
    current_chr: Option<char>,
    previous_chr: Option<char>,
    row: usize,
    column: usize,
    identifiers: HashMap<String, Token>,
    operators: Vec<char>,
    delimiters: Vec<char>,
    current_char_processed: bool,
}

fn get_identifiers_map() -> HashMap<String, Token> {
    let mut identifiers: HashMap<String, Token> = HashMap::new();

    // Keywords
    identifiers.insert(String::from("if"), Token::If);
    identifiers.insert(String::from("elif"), Token::Elif);
    identifiers.insert(String::from("else"), Token::Else);
    identifiers.insert(String::from("for"), Token::For);
    identifiers.insert(String::from("while"), Token::While);
    identifiers.insert(String::from("class"), Token::Class);
    identifiers.insert(String::from("fn"), Token::Function);
    identifiers.insert(String::from("priv"), Token::Private);
    identifiers.insert(String::from("pub"), Token::Public);
    identifiers.insert(String::from("new"), Token::NewInstance);
    identifiers.insert(String::from("self"), Token::SelfInstance);
    identifiers.insert(String::from("del"), Token::DelObject);
    identifiers.insert(String::from("construct"), Token::Constructor);
    identifiers.insert(String::from("destruct"), Token::Destructor);
    identifiers.insert(String::from("super"), Token::Super);
    identifiers.insert(String::from("return"), Token::Return);

    // Literal values
    identifiers.insert(String::from("true"), Token::BoolValue { value: true });
    identifiers.insert(String::from("false"), Token::BoolValue { value: false });
    identifiers.insert(String::from("null"), Token::NullValue);

    // Builtin types
    identifiers.insert(String::from("int"), Token::IntType);
    identifiers.insert(String::from("float"), Token::FloatType);
    identifiers.insert(String::from("str"), Token::StringType);
    identifiers.insert(String::from("char"), Token::CharType);
    identifiers.insert(String::from("bool"), Token::BoolType);
    identifiers.insert(String::from("bytes"), Token::BytesType);
    identifiers.insert(String::from("tuple"), Token::TupleType);
    identifiers.insert(String::from("enum"), Token::EnumType);
    identifiers.insert(String::from("list"), Token::ListType);
    identifiers.insert(String::from("dict"), Token::DictType);

    identifiers
}

fn get_operators() -> Vec<char> {
    vec![
        '+', '-', '*', '/', '%', '!', '=', '|', '&', '^', '<', '>', '~',
    ]
}

fn get_delimiters() -> Vec<char> {
    vec![
        '{', '}', '[', ']', '(', ')', ',', ';', ':', '.', 
    ]
}

impl<T> Lexer<T>
where
    T: Iterator<Item = char>,
{
    pub fn new(input: T) -> Self {
        Lexer {
            input: input.peekable(),
            current_chr: None,
            previous_chr: None,
            row: 0,
            column: 0,
            identifiers: get_identifiers_map(),
            operators: get_operators(),
            delimiters: get_delimiters(),
            current_char_processed: true
        }
    }

    // todo: implement as iterator
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        if self.current_char_processed {
            self.next_char();
        }
        else {
            self.current_char_processed = true;
        }
        
        self.skip_redundant_characters();

        if self.current_chr.is_none() {
            return Err(LexerError {
                message: String::from("No more tokens"),
            });
        }

        if self.is_letter() || self.char_equals(UNDERSCORE) {
            return self.handle_identifier();
        }

        if self.is_digit() {
            return self.handle_number();
        }

        if self.is_beginning_of_string() {
            return self.handle_string();
        }

        if self.is_beginning_of_char() {
            return self.handle_char();
        }

        if self.is_operator() {
            return self.handle_operator();
        }

        if self.is_delimiter() {
            return self.handle_delimiter();
        }

        Err(LexerError {
            message: String::from("Failed to lex source"),
        })
    }

    fn next_char(&mut self) {
        self.previous_chr = self.current_chr;
        self.current_chr = self.input.next();

        if self.current_chr.is_some() {
            self.column += 1;
        }
    }

    fn is_whitespace(&self) -> bool {
        self.current_chr.unwrap().is_whitespace()
    }

    fn is_newline(&mut self) -> bool {
        return match self.current_chr {
            Some('\n') => true,
            Some('\r') => {
                return match self.input.peek() {
                    Some('\n') => {
                        self.next_char();
                        return true;
                    }
                    _ => false,
                };
            }
            _ => false,
        };
    }

    fn is_alphanumeric(&self) -> bool {
        self.current_chr.unwrap().is_ascii_alphanumeric()
    }

    fn is_letter(&self) -> bool {
        self.current_chr.unwrap().is_ascii_alphabetic()
    }

    fn is_beginning_of_string(&self) -> bool {
        self.char_equals(DOUBLE_QUOTES)
    }

    fn is_beginning_of_char(&self) -> bool {
        self.char_equals(SINGLE_QUOTES)
    }

    fn is_digit(&self) -> bool {
        self.current_chr.unwrap().is_ascii_digit()
    }

    fn is_operator(&self) -> bool {
        self.operators.contains(&self.current_chr.unwrap())
    }

    fn is_delimiter(&self) -> bool {
        self.delimiters.contains(&self.current_chr.unwrap())
    }

    fn char_equals(&self, compared_char: char) -> bool {
        self.current_chr.unwrap() == compared_char
    }

    fn skip_redundant_characters(&mut self) {
        while self.current_chr.is_some() && (self.is_whitespace() || self.is_newline()) {
            if self.is_newline() {
                self.row += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }

            self.next_char();
        }
    }

    fn handle_identifier(&mut self) -> Result<Token, LexerError> {
        let mut identifier = String::from("");

        // Loop until end of word
        while self.current_chr.is_some() && (self.is_alphanumeric() || self.char_equals(UNDERSCORE)) {
            identifier.push(self.current_chr.unwrap());
            self.next_char();
        }

        self.current_char_processed = false;

        // Common identifiers (e.g: "if", "true", "int", "while", ...)
        if self.identifiers.contains_key(&identifier) {
            return Ok(self.identifiers.get(&identifier).unwrap().clone());
        }
        // Literal bytes value (i.e: b"h\x04\x12")
        else if identifier.len() == 1
            && self.previous_chr.unwrap() == BYTES_PREFIX
            && self.current_chr.is_some()
            && self.char_equals(DOUBLE_QUOTES)
        {
            self.current_char_processed = true;

            identifier = String::from(""); // Reset identifier (i.e, remove the 'b' character).

            self.next_char();

            while self.current_chr.is_some() && !self.char_equals(DOUBLE_QUOTES) {
                identifier.push(self.current_chr.unwrap());
                self.next_char();
            }

            if !self.char_equals(DOUBLE_QUOTES) {
                return Err(LexerError {
                    message: String::from("Failed to parse bytes value: missing double-quotes"),
                });
            }

            self.next_char();
            self.current_char_processed = false;

            return Ok(Token::BytesValue {
                value: identifier.as_bytes().to_vec(),
            });
        }
        // Symbol names
        else {
            return Ok(Token::Symbol { name: identifier });
        }
    }

    fn handle_number(&mut self) -> Result<Token, LexerError> {
        let mut number = String::from("");

        while self.current_chr.is_some() && (self.is_digit() || self.char_equals(DOT_SEPERATOR)) {
            number.push(self.current_chr.unwrap());
            self.next_char();
        }

        self.current_char_processed = false;

        return match number.matches(DOT_SEPERATOR).count() {
            1 => {
                let parsed_number = number.parse::<f64>();

                if parsed_number.is_err() {
                    return Err(LexerError {
                        message: String::from("Could not parse float"),
                    });
                }

                Ok(Token::FloatValue {
                    value: parsed_number.unwrap(),
                })
            },
            0 => {
                let parsed_number = BigInt::from_str(&number);

                if parsed_number.is_err() {
                    return Err(LexerError {
                        message: String::from("Could not parse int"),
                    });
                }

                Ok(Token::IntValue {
                    value: parsed_number.unwrap(),
                })
            },
            _ => Err(LexerError {
                message: String::from("Invalid number - too many dot seperators"),
            })
        };
    }

    fn handle_string(&mut self) -> Result<Token, LexerError> {
        let mut string = String::from("");

        self.next_char();

        while self.current_chr.is_some() && !self.char_equals(DOUBLE_QUOTES) {
            string.push(self.current_chr.unwrap());
            self.next_char();
        }

        if !self.char_equals(DOUBLE_QUOTES) {
            return Err(LexerError {
                message: String::from("Failed to parse string value: missing double-quotes"),
            });
        }

        return Ok(Token::StringValue {
            value: string
        });
    }

    fn handle_char(&mut self) -> Result<Token, LexerError> {
        self.next_char();

        if self.current_chr.is_none() {
            return Err(LexerError {
                message: String::from("Failed to parse character value"),
            });
        }
        else if self.current_chr.is_some() && self.char_equals(SINGLE_QUOTES) {
            return Err(LexerError {
                message: String::from("Character literal may only contain one codepoint"),
            });
        }

        let chr = self.current_chr.unwrap();
        
        self.next_char(); 

        if self.current_chr.is_none() || (self.current_chr.is_some() && !self.char_equals(SINGLE_QUOTES)) {
            return Err(LexerError {
                message: String::from("Failed to parse character value: missing single-quotes"),
            });
        }
        
        return Ok(Token::CharValue {
            value: chr
        });
    }

    fn handle_operator(&mut self) -> Result<Token, LexerError> {
        return match self.current_chr.unwrap() {
            '+' => Ok(Token::Add),
            '-' => {
                return match self.input.peek() {
                    Some('>') => {
                        self.next_char();
                        return Ok(Token::FnReturnTypeDelim);
                    },
                    _ => Ok(Token::Subtract)
                }
            },
            '*' => Ok(Token::Multiply),
            '/' => Ok(Token::Divide),
            '%' => Ok(Token::Modulo),
            '!' => {
                return match self.input.peek() {
                    Some('=') => {
                        self.next_char();
                        return Ok(Token::NotEquals);
                    },
                    _ => Ok(Token::Not)
                }
            },
            '=' => {
                return match self.input.peek() {
                    Some('=') => {
                        self.next_char();
                        return Ok(Token::Equals);
                    },
                    _ => Ok(Token::Assignment)
                }
            },
            '|' => {
                return match self.input.peek() {
                    Some('|') => {
                        self.next_char();
                        return Ok(Token::LogicalOr);
                    },
                    _ => Ok(Token::BitwiseOr)
                }
            },
            '&' => {
                return match self.input.peek() {
                    Some('&') => {
                        self.next_char();
                        return Ok(Token::LogicalAnd);
                    },
                    _ => Ok(Token::BitwiseAnd)
                }
            },
            '~' => Ok(Token::BitwiseNot),
            '^' => Ok(Token::BitwiseXor),
            '>' => {
                return match self.input.peek() {
                    Some('>') => {
                        self.next_char();
                        return Ok(Token::BitwiseRightShift);
                    },
                    Some('=') => {
                        self.next_char();
                        return Ok(Token::GreaterEqual);
                    },
                    _ => Ok(Token::Greater)
                }
            },
            '<' => {
                return match self.input.peek() {
                    Some('<') => {
                        self.next_char();
                        return Ok(Token::BitwiseLeftShift);
                    },
                    Some('=') => {
                        self.next_char();
                        return Ok(Token::LessEqual);
                    },
                    _ => Ok(Token::Less)
                }
            },
            _ => Err(LexerError {
                message: String::from("Could not parse operator"),
            })
        };
    }

    fn handle_delimiter(&mut self) -> Result<Token, LexerError> {
        return match self.current_chr.unwrap() {
            '(' => Ok(Token::LeftParens),
            ')' => Ok(Token::RightParens),
            '{' => Ok(Token::LeftCurlyBracket),
            '}' => Ok(Token::RightCurlyBracket),
            '[' => Ok(Token::LeftSquareBracket),
            ']' => Ok(Token::RightSquareBracket),
            ';' => Ok(Token::Semicolon),
            ',' => Ok(Token::Comma),
            '.' => Ok(Token::MemberAccessor),
            '-' => {
                return match self.input.peek() {
                    Some('>') => {
                        self.next_char();
                        return Ok(Token::FnReturnTypeDelim);
                    },
                    _ => Err(LexerError {
                        message: String::from("Could not parse delimiter"),
                    })
                }
            },
            ':' => {
                return match self.input.peek() {
                    Some(':') => {
                        self.next_char();
                        return Ok(Token::StaticAccessor);
                    },
                    _ => Err(LexerError {
                        message: String::from("Could not parse delimiter"),
                    })
                }
            },
            _ => Err(LexerError {
                message: String::from("Could not parse delimiter"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::errors::LexerError;
    use crate::parser::lexer::Lexer;
    use crate::parser::token::Token;
    use num_bigint::BigInt;

    pub fn lex_source(source: &String) -> Vec<Token> {
        let mut lexer = Lexer::new(source.chars());
        let mut tokens: Vec<Token> = Vec::new();
        let mut token: Result<Token, LexerError> = lexer.next_token();

        while token.is_ok() {
            tokens.push(token.unwrap());
            token = lexer.next_token();
        }

        tokens
    }

    #[test]
    fn test_class_structure() {
        let source = String::from(r#"
            class Logger {
                priv str name;

                fn construct(str name) {
                    self.name = name;
                }
                
                fn destruct() {
                    int i = 76;
                }

                fn clone() -> Logger;
            }
        "#);
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![
                Token::Class,
                Token::Symbol { name: String::from("Logger") },
                Token::LeftCurlyBracket,
                Token::Private,
                Token::StringType,
                Token::Symbol { name: String::from("name") },
                Token::Semicolon,
                Token::Function,
                Token::Constructor,
                Token::LeftParens,
                Token::StringType,
                Token::Symbol { name: String::from("name") },
                Token::RightParens,
                Token::LeftCurlyBracket,
                Token::SelfInstance,
                Token::MemberAccessor,
                Token::Symbol { name: String::from("name") },
                Token::Assignment,
                Token::Symbol { name: String::from("name") },
                Token::Semicolon,
                Token::RightCurlyBracket,
                Token::Function,
                Token::Destructor,
                Token::LeftParens,
                Token::RightParens,
                Token::LeftCurlyBracket,
                Token::IntType,
                Token::Symbol { name: String::from("i") },
                Token::Assignment,
                Token::IntValue { value: BigInt::from(76) },
                Token::Semicolon,
                Token::RightCurlyBracket,
                Token::Function,
                Token::Symbol { name: String::from("clone") },
                Token::LeftParens,
                Token::RightParens,
                Token::FnReturnTypeDelim,
                Token::Symbol { name: String::from("Logger") },
                Token::Semicolon,
                Token::RightCurlyBracket
            ]
        );
    }

    #[test]
    fn test_assignment_expressions() {
        let source = String::from(r#"
            int i = 5;
            float f = 3.54;
            bool b = true;
            str s1 = "Hello World";
            bytes bb = b"\x34b";
            list l = [i, f, b, s1, "some_val"];
        "#);
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![
                Token::IntType,
                Token::Symbol { name: String::from("i") },
                Token::Assignment,
                Token::IntValue { value: BigInt::from(5) },
                Token::Semicolon,
                Token::FloatType,
                Token::Symbol { name: String::from("f") },
                Token::Assignment,
                Token::FloatValue { value: 3.54 },
                Token::Semicolon,
                Token::BoolType,
                Token::Symbol { name: String::from("b") },
                Token::Assignment,
                Token::BoolValue { value: true },
                Token::Semicolon,
                Token::StringType,
                Token::Symbol { name: String::from("s1") },
                Token::Assignment,
                Token::StringValue { value: String::from("Hello World") },
                Token::Semicolon,
                Token::BytesType,
                Token::Symbol { name: String::from("bb") },
                Token::Assignment,
                Token::BytesValue { value: String::from(r#"\x34b"#).as_bytes().to_vec() },
                Token::Semicolon,
                Token::ListType,
                Token::Symbol { name: String::from("l") },
                Token::Assignment,
                Token::LeftSquareBracket,
                Token::Symbol { name: String::from("i") },
                Token::Comma,
                Token::Symbol { name: String::from("f") },
                Token::Comma,
                Token::Symbol { name: String::from("b") },
                Token::Comma,
                Token::Symbol { name: String::from("s1") },
                Token::Comma,
                Token::StringValue { value: String::from("some_val") },
                Token::RightSquareBracket,
                Token::Semicolon
            ]
        );
    }

    #[test]
    fn test_variable_identifiers() {
        let source =
            String::from("str some_str int i float _ff bytes ba char c tuple t list lll dict d enum e");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![
                Token::StringType,
                Token::Symbol {
                    name: String::from("some_str")
                },
                Token::IntType,
                Token::Symbol {
                    name: String::from("i")
                },
                Token::FloatType,
                Token::Symbol {
                    name: String::from("_ff")
                },
                Token::BytesType,
                Token::Symbol {
                    name: String::from("ba")
                },
                Token::CharType,
                Token::Symbol {
                    name: String::from("c")
                },
                Token::TupleType,
                Token::Symbol {
                    name: String::from("t")
                },
                Token::ListType,
                Token::Symbol {
                    name: String::from("lll")
                },
                Token::DictType,
                Token::Symbol {
                    name: String::from("d")
                },
                Token::EnumType,
                Token::Symbol {
                    name: String::from("e")
                },
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let source = String::from(r#""Some string value""#);
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![Token::StringValue {
                value: String::from("Some string value")
            },]
        );
    }

    #[test]
    fn test_bytes_literal() {
        let source = String::from(r#"b"hello \x01\03 \x44""#);
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![Token::BytesValue {
                value: String::from(r#"hello \x01\03 \x44"#).as_bytes().to_vec()
            },]
        );
    }

    #[test]
    fn test_character_literal() {
        let source = String::from("'a'");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![Token::CharValue {
                value: 'a'
            },]
        );
    }

    #[test]
    fn test_number_literal() {
        let source = String::from("423 763.433 0 24454333");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![
                Token::IntValue {
                    value: BigInt::from(423)
                },
                Token::FloatValue { value: 763.433 },
                Token::IntValue {
                    value: BigInt::from(0)
                },
                Token::IntValue {
                    value: BigInt::from(24454333)
                },
            ]
        );
    }

    #[test]
    fn test_operators() {
        let source = String::from("|| && + - * / % | ^ ~ & >> << ! == != > >= < <= =");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![
                Token::LogicalOr,
                Token::LogicalAnd,
                Token::Add,
                Token::Subtract,
                Token::Multiply,
                Token::Divide,
                Token::Modulo,
                Token::BitwiseOr,
                Token::BitwiseXor,
                Token::BitwiseNot,
                Token::BitwiseAnd,
                Token::BitwiseRightShift,
                Token::BitwiseLeftShift,
                Token::Not,
                Token::Equals,
                Token::NotEquals,
                Token::Greater,
                Token::GreaterEqual,
                Token::Less,
                Token::LessEqual,
                Token::Assignment
            ]
        );
    }

    #[test]
    fn test_delimiters() {
        let source = String::from("( ) { } [ ] . ; , :: ->");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens,
            vec![
                Token::LeftParens,
                Token::RightParens,
                Token::LeftCurlyBracket,
                Token::RightCurlyBracket,
                Token::LeftSquareBracket,
                Token::RightSquareBracket,
                Token::MemberAccessor,
                Token::Semicolon,
                Token::Comma,
                Token::StaticAccessor,
                Token::FnReturnTypeDelim
            ]
        );
    }
}
