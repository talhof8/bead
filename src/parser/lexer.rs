use std::iter::Peekable;
use std::collections::HashMap;
use std::str::FromStr;
use num_bigint::BigInt;
use crate::parser::errors::LexerError;
use crate::parser::token::Token;

const SEMICOLON: char = ';';
const DOUBLE_QUOTES: char = '"';
const BYTES_PREFIX: char = 'b';
const DOT_SEPERATOR: char = '.';

pub struct Lexer<T: Iterator<Item = char>> { 
    input: Peekable<T>,
    current_chr: Option<char>,
    previous_chr: Option<char>,
    row: usize,
    column: usize,
    identifiers: HashMap<String, Token>,
    operators: Vec<char>
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
    identifiers.insert(String::from("construct"), Token::Constructor);
    identifiers.insert(String::from("destruct"), Token::Destructor);
    identifiers.insert(String::from("super"), Token::Super);
    identifiers.insert(String::from("return"), Token::Return);

    // Literal values
    identifiers.insert(String::from("true"), Token::True);
    identifiers.insert(String::from("false"), Token::False);
    identifiers.insert(String::from("null"), Token::Null);

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
    vec!['+', '-', '*', '/', '%', '!', '=', '|', '&', '^', '<', '>', '~']
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
            operators: get_operators()
        }
    }

    // todo: implement as iterator
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.next_char();

        if self.current_chr.is_none() {
            return Err(LexerError { message: String::from("No more tokens") });
        }

        self.skip_redundant_characters();

        if self.is_letter() {
            return self.handle_identifier();
        }

        if self.is_digit() {
            return self.handle_number();
        }

        if self.is_operator() {
            return self.handle_operator();
        }

        Err(LexerError { message: String::from("Failed to lex source") })
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
                    },
                    _ => false
                };
            },
            _ => false
        };
    }

    fn is_alphanumeric(&self) -> bool {
        self.current_chr.unwrap().is_ascii_alphanumeric()
    }

    fn is_letter(&self) -> bool {
        self.current_chr.unwrap().is_ascii_alphabetic()
    }

    fn is_digit(&self) -> bool {
        self.current_chr.unwrap().is_ascii_digit()
    }

    fn is_operator(&self) -> bool {
        self.operators.contains(&self.current_chr.unwrap())
    }
    
    fn char_equals(&self, compared_char: char) -> bool {
        self.current_chr.unwrap() == compared_char
    }

    fn skip_redundant_characters(&mut self) {
        while self.current_chr.is_some() && (self.is_whitespace() || self.is_newline()) {
            if self.is_newline() {
                self.row += 1;
                self.column = 0; 
            }
            else {
                self.column += 1;
            }

            self.next_char();
        }
    }

    fn handle_identifier(&mut self) -> Result<Token, LexerError> {
        let mut identifier = String::from("");

        // Loop until end of word
        while self.current_chr.is_some() && self.is_alphanumeric() { 
            identifier.push(self.current_chr.unwrap());
            self.next_char();
        }
        
        // Common identifiers (e.g: "if", "true", "int", "while", ...)
        if self.identifiers.contains_key(&identifier) {
            return Ok(self.identifiers.get(&identifier).unwrap().clone());
        }
        // Literal bytes values (i.e: b"...")
        else if identifier.len() == 1 && self.previous_chr.unwrap() == BYTES_PREFIX 
            && self.current_chr.is_some() && self.char_equals(DOUBLE_QUOTES) {
            // Identifier needs to include the left double-quotes as well.
            identifier.push(self.current_chr.unwrap());
            self.next_char();
            
            while self.current_chr.is_some() && !self.char_equals(DOUBLE_QUOTES) {
                identifier.push(self.current_chr.unwrap());
                self.next_char();
            }

            if !self.char_equals(DOUBLE_QUOTES) {
                return Err(LexerError { message: String::from("Failed to parse bytes value: missing double-quotes") });
            }
            else {
                // Name needs to include the right double-quotes as well.
                identifier.push(self.current_chr.unwrap());
                self.next_char();

                return Ok(Token::BytesValue { value: identifier.as_bytes().to_vec() });
            }
        }
        // Symbol names
        else {
            return Ok(Token::Symbol { name: identifier });
        }
    }

    fn handle_number(&mut self) ->  Result<Token, LexerError> {
        let mut number = String::from("");

        while self.current_chr.is_some() && (self.is_digit() || self.char_equals(DOT_SEPERATOR)) {
            number.push(self.current_chr.unwrap());
            self.next_char();
        }

        if self.current_chr.is_some() && self.is_letter() {
            return Err(LexerError { message: String::from("Number literal cannot end with a letter") });
        }

        return match number.matches(DOT_SEPERATOR).count() {
            1 => {
                let parsed_number = number.parse::<f64>();

                if parsed_number.is_err() {
                    return Err(LexerError { message: String::from("Could not parse float") })
                }

                Ok(Token::FloatValue { value: parsed_number.unwrap() })
            },
            0 => {
                let parsed_number = BigInt::from_str(&number);

                if parsed_number.is_err() {
                    return Err(LexerError { message: String::from("Could not parse int") })
                }

                Ok(Token::IntValue { value: parsed_number.unwrap() })
            },
            _ => Err(LexerError { message: String::from("Invalid number - too many dot seperators") })
        };
    }

    fn handle_operator(&mut self) -> Result<Token, LexerError> {
        return match self.current_chr.unwrap() {
            '+' => Ok(Token::Add),
            '-' => Ok(Token::Subtract),
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
            _=> Err(LexerError { message: String::from("Could not parse operator") })
        };
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use crate::parser::lexer::Lexer;
    use crate::parser::token::Token;
    use crate::parser::errors::LexerError;

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
    fn test_variable_identifiers() {
        let source = String::from("str s1 int i float ff bytes ba char c tuple t list lll dict d enum e");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens, 
            vec![
                Token::StringType, 
                Token::Symbol { name: String::from("s1") },
                Token::IntType, 
                Token::Symbol { name: String::from("i") },
                Token::FloatType, 
                Token::Symbol { name: String::from("ff") },
                Token::BytesType, 
                Token::Symbol { name: String::from("ba") },
                Token::CharType, 
                Token::Symbol { name: String::from("c") },
                Token::TupleType, 
                Token::Symbol { name: String::from("t") },
                Token::ListType, 
                Token::Symbol { name: String::from("lll") },
                Token::DictType, 
                Token::Symbol { name: String::from("d") },
                Token::EnumType, 
                Token::Symbol { name: String::from("e") },
            ]
        );
    }

    #[test]
    fn test_bytes_literal() {
        let source = String::from(r#"b"hello \x01\03 \x44""#);
        let tokens = lex_source(&source);
        assert_eq!(
            tokens, 
            vec![
                Token::BytesValue { value: String::from(r#"b"hello \x01\03 \x44""#).as_bytes().to_vec() },
            ]
        );
    }

    #[test]
    fn test_number_literal() {
        let source = String::from("423 763.433 0 24454333");
        let tokens = lex_source(&source);
        assert_eq!(
            tokens, 
            vec![
                Token::IntValue { value: BigInt::from(423) },
                Token::FloatValue { value: 763.433 },
                Token::IntValue { value: BigInt::from(0) },
                Token::IntValue { value: BigInt::from(24454333) },
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
}