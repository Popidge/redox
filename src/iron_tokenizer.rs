//! Iron language tokenizer
//!
//! Tokenizes Iron source code into tokens for parsing.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Function,
    With,
    Generic,
    Type,
    Implementing,
    Takes,
    Returns,
    Begin,
    End,
    Define,
    Mutable,
    As,
    Set,
    Equal,
    To,
    If,
    Condition,
    Then,
    Otherwise,
    Compare,
    Case,
    While,
    Repeat,
    For,
    Each,
    In,
    Iterator,
    Loop,
    Forever,
    Exit,
    Continue,
    Return,
    Structure,
    Fields,
    Field,
    Enumeration,
    Variants,
    Variant,
    Of,
    Reference,
    Raw,
    Pointer,
    Optional,
    Result,
    List,
    Box,
    Call,
    Method,
    On,
    Associated,
    Constant,
    Static,
    Macro,
    Bracket,
    Comma,
    Context,
    Some,
    None,
    Ok,
    Error,
    Note,
    That,
    Unwrap,
    Or,
    And,
    Plus,
    Minus,
    Times,
    Divided,
    By,
    Modulo,
    Less,
    Greater,
    Than,
    Not,
    Tuple,
    Array,
    Slice,
    Containing,
    Create,
    Index,
    At,
    Range,
    From,
    Inclusive,
    Taking,
    Returning,
    Closure,
    Move,
    Parameters,
    Body,
    Pipe,
    Verbatim,

    // Literals
    Identifier(String),
    String(String),
    Integer(String),
    Float(String),
    Boolean(bool),

    // Special
    NewLine,
    Indent(usize),
    EndOfFile,
}

pub struct Tokenizer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut _current_indent = 0;

        while self.position < self.input.len() {
            let ch = self.current_char();

            // Handle newlines and indentation
            if ch == '\n' {
                tokens.push(Token::NewLine);
                self.advance();

                // Count indentation on next line
                let mut indent = 0;
                while self.position < self.input.len() && self.current_char() == ' ' {
                    indent += 1;
                    self.advance();
                }

                // Only track indentation if there's actual content
                if self.position < self.input.len() && self.current_char() != '\n' {
                    tokens.push(Token::Indent(indent));
                    let _ = indent;
                }
                continue;
            }

            // Skip regular whitespace
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // Skip comments (note that ...)
            if self.starts_with("note that") {
                while self.position < self.input.len() && self.current_char() != '\n' {
                    self.advance();
                }
                continue;
            }

            // String literals
            if ch == '"' {
                tokens.push(self.read_string());
                continue;
            }

            // Character literals
            if ch == '\'' {
                tokens.push(self.read_char());
                continue;
            }

            // Numbers
            if ch.is_ascii_digit() {
                tokens.push(self.read_number());
                continue;
            }

            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' {
                tokens.push(self.read_word());
                continue;
            }

            // Handle punctuation
            match ch {
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                    continue;
                }
                _ => {}
            }

            // Unknown character - skip
            self.advance();
        }

        tokens.push(Token::EndOfFile);
        tokens
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // skip opening quote
        let mut value = String::new();

        while self.position < self.input.len() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                match self.current_char() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    c => value.push(c),
                }
            } else {
                value.push(self.current_char());
            }
            self.advance();
        }

        if self.current_char() == '"' {
            self.advance(); // skip closing quote
        }

        Token::String(value)
    }

    fn read_char(&mut self) -> Token {
        self.advance(); // skip opening quote
        let mut value = String::new();

        while self.position < self.input.len() && self.current_char() != '\'' {
            value.push(self.current_char());
            self.advance();
        }

        if self.current_char() == '\'' {
            self.advance(); // skip closing quote
        }

        Token::String(value) // Represent as string for simplicity
    }

    fn read_number(&mut self) -> Token {
        let mut value = String::new();
        let mut is_float = false;

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                // Check if next char is digit (to distinguish from method call)
                let next_pos = self.position + 1;
                if next_pos < self.input.len()
                    && self.input.chars().nth(next_pos).unwrap().is_ascii_digit()
                {
                    is_float = true;
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(value)
        } else {
            Token::Integer(value)
        }
    }

    fn read_word(&mut self) -> Token {
        let mut word = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        self.match_keyword(&word)
    }

    fn match_keyword(&self, word: &str) -> Token {
        // Check for multi-word keywords first
        let remaining = &self.input[self.position..];

        match word {
            "function" => Token::Function,
            "with" => {
                if remaining.starts_with(" fields") || self.check_context() == "struct" {
                    Token::With
                } else if remaining.starts_with(" generic") {
                    Token::With
                } else {
                    Token::With
                }
            }
            "generic" => Token::Generic,
            "type" => {
                if remaining.starts_with(" T") || remaining.starts_with(" ") {
                    // Check if this is "type T" in a generic context
                    Token::Type
                } else {
                    Token::Type
                }
            }
            "implementing" => Token::Implementing,
            "takes" => Token::Takes,
            "returns" => Token::Returns,
            "begin" => Token::Begin,
            "end" => {
                // Check following word for end block type
                if remaining.starts_with(" function") {
                    Token::End
                } else if remaining.starts_with(" if") {
                    Token::End
                } else if remaining.starts_with(" for") {
                    Token::End
                } else if remaining.starts_with(" while") {
                    Token::End
                } else if remaining.starts_with(" structure") {
                    Token::End
                } else if remaining.starts_with(" enumeration") {
                    Token::End
                } else {
                    Token::End
                }
            }
            "define" => Token::Define,
            "mutable" => Token::Mutable,
            "as" => Token::As,
            "set" => Token::Set,
            "equal" => Token::Equal,
            "to" => Token::To,
            "if" => Token::If,
            "condition" => Token::Condition,
            "then" => Token::Then,
            "otherwise" => Token::Otherwise,
            "compare" => Token::Compare,
            "case" => Token::Case,
            "while" => Token::While,
            "repeat" => Token::Repeat,
            "for" => {
                if remaining.starts_with(" each") {
                    Token::For
                } else {
                    Token::For
                }
            }
            "each" => Token::Each,
            "in" => Token::In,
            "iterator" => Token::Iterator,
            "loop" => Token::Loop,
            "forever" => Token::Forever,
            "exit" => Token::Exit,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "structure" => Token::Structure,
            "fields" => Token::Fields,
            "Fields" => Token::Fields,
            "field" => Token::Field,
            "enumeration" => Token::Enumeration,
            "variants" => Token::Variants,
            "variant" => Token::Variant,
            "of" => Token::Of,
            "reference" => Token::Reference,
            "raw" => Token::Raw,
            "pointer" => Token::Pointer,
            "optional" => Token::Optional,
            "result" => Token::Result,
            "list" => Token::List,
            "box" => Token::Box,
            "call" => Token::Call,
            "method" => Token::Method,
            "macro" => Token::Macro,
            "bracket" => Token::Bracket,
            "on" => Token::On,
            "associated" => Token::Associated,
            "constant" => Token::Constant,
            "static" => Token::Static,
            "context" => Token::Context,
            "some" => Token::Some,
            "none" => Token::None,
            "ok" => Token::Ok,
            "error" => Token::Error,
            "note" => Token::Note,
            "that" => Token::That,
            "unwrap" => Token::Unwrap,
            "or" => Token::Or,
            "and" => Token::And,
            "plus" => Token::Plus,
            "minus" => Token::Minus,
            "times" => Token::Times,
            "divided" => Token::Divided,
            "by" => Token::By,
            "modulo" => Token::Modulo,
            "less" => Token::Less,
            "greater" => Token::Greater,
            "than" => Token::Than,
            "not" => Token::Not,
            "tuple" => Token::Tuple,
            "array" => Token::Array,
            "slice" => Token::Slice,
            "containing" => Token::Containing,
            "create" => Token::Create,
            "index" => Token::Index,
            "at" => Token::At,
            "range" => Token::Range,
            "from" => Token::From,
            "inclusive" => Token::Inclusive,
            "taking" => Token::Taking,
            "returning" => Token::Returning,
            "closure" => Token::Closure,
            "move" => Token::Move,
            "parameters" => Token::Parameters,
            "body" => Token::Body,
            "verbatim" => Token::Verbatim,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => {
                // Check for user_ prefix (collision-avoiding identifier)
                if word.starts_with("user_") {
                    Token::Identifier(word[5..].to_string())
                } else {
                    Token::Identifier(word.to_string())
                }
            }
        }
    }

    fn check_context(&self) -> &str {
        // Simple context checking - can be improved
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let input = "function hello\nbegin\nend function";
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize();

        assert!(tokens.contains(&Token::Function));
        assert!(tokens.contains(&Token::Identifier("hello".to_string())));
        assert!(tokens.contains(&Token::Begin));
        assert!(tokens.contains(&Token::End));
    }

    #[test]
    fn test_tokenize_user_prefix() {
        let input = "define user_function as 42";
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize();

        let id_token = tokens.iter().find(|t| matches!(t, Token::Identifier(_)));
        assert!(matches!(id_token, Some(Token::Identifier(name)) if name == "function"));
    }
}
