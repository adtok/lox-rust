use std::collections::HashMap;

fn is_digit(ch: char) -> bool {
    let uch = ch as u8;
    uch >= '0' as u8 && uch <= '9' as u8
}

fn is_alpha(ch: char) -> bool {
    let uch = ch as u8;
    (uch >= 'a' as u8 && uch <= 'z' as u8) || (uch >= 'A' as u8 && uch <= 'Z' as u8) || (ch == '_')
}

fn is_alphanumeric(ch: char) -> bool {
    is_alpha(ch) || is_digit(ch)
}

fn get_keywords_hashmap() -> HashMap<&'static str, TokenType> {
    HashMap::from([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fun", TokenType::Fun),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ])
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            keywords: get_keywords_hashmap(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut errors = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(msg) => errors.push(msg),
            }
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::from(""),
            literal: None,
            line: self.line,
        });

        if errors.len() > 0 {
            let mut joined = String::new();
            for error in errors {
                joined.push_str(&error);
                joined.push_str("\n");
            }
            return Err(joined);
        }

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.char_match('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.char_match('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.char_match('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.char_match('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.char_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.char_match('*') {
                    todo!("Add support for multiline comments")
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            c => {
                if is_digit(c) {
                    self.number()?
                } else if is_alpha(c) {
                    self.identifier()
                } else {
                    return Err(format!("Unrecognized char at line {}: {}", self.line, c));
                }
            }
        }

        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;

        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_lit(token_type, None);
    }

    fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = self.source[self.start..self.current].to_string();

        self.tokens.push(Token {
            token_type: token_type,
            lexeme: text,
            literal: literal,
            line: self.line,
        })
    }

    fn char_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];

        self.add_token_lit(
            TokenType::StringLit,
            Some(LiteralValue::StringValue(value.to_string())),
        );

        Ok(())
    }

    fn number(&mut self) -> Result<(), String> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }
        let substring = &self.source[self.start..self.current];
        let value = substring.parse::<f64>();

        match value {
            Ok(value) => self.add_token_lit(TokenType::Number, Some(LiteralValue::FValue(value))),
            Err(_) => return Err(format!("Could not parse number: {}", substring)),
        }

        Ok(())
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let substring = &self.source[self.start..self.current];

        if let Some(&t_type) = self.keywords.get(substring) {
            self.add_token(t_type);
        } else {
            self.add_token(TokenType::Identifier);
        }
    }
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    FValue(f64),
    StringValue(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    StringLit,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_one_char_tokens() {
        let source = "() {} ,. -+ ; / *";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 12);

        assert_eq!(scanner.tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(scanner.tokens[1].token_type, TokenType::RightParen);
        assert_eq!(scanner.tokens[2].token_type, TokenType::LeftBrace);
        assert_eq!(scanner.tokens[3].token_type, TokenType::RightBrace);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Comma);
        assert_eq!(scanner.tokens[5].token_type, TokenType::Dot);
        assert_eq!(scanner.tokens[6].token_type, TokenType::Minus);
        assert_eq!(scanner.tokens[7].token_type, TokenType::Plus);
        assert_eq!(scanner.tokens[8].token_type, TokenType::Semicolon);
        assert_eq!(scanner.tokens[9].token_type, TokenType::Slash);
        assert_eq!(scanner.tokens[10].token_type, TokenType::Star);
        assert_eq!(scanner.tokens[11].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_two_char_tokens() {
        let source = "! != = == > >= < <=";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 9);

        assert_eq!(scanner.tokens[0].token_type, TokenType::Bang);
        assert_eq!(scanner.tokens[1].token_type, TokenType::BangEqual);
        assert_eq!(scanner.tokens[2].token_type, TokenType::Equal);
        assert_eq!(scanner.tokens[3].token_type, TokenType::EqualEqual);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Greater);
        assert_eq!(scanner.tokens[5].token_type, TokenType::GreaterEqual);
        assert_eq!(scanner.tokens[6].token_type, TokenType::Less);
        assert_eq!(scanner.tokens[7].token_type, TokenType::LessEqual);
        assert_eq!(scanner.tokens[8].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_string_literal() {
        let source = "\"Hello, world!\"";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 2);

        assert_eq!(scanner.tokens[0].token_type, TokenType::StringLit);
        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::StringValue(s) => assert_eq!(s, "Hello, world!"),
            _ => panic!("Incorrect LiteralValue type"),
        }
        assert_eq!(scanner.tokens[1].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_unterminated_string_literal() {
        let source = "\"Hello, world!";
        let mut scanner = Scanner::new(source);

        match scanner.scan_tokens() {
            Err(_) => (),
            _ => panic!("Should have failed scanning."),
        }
    }

    #[test]
    fn scan_multiline_string_literal() {
        let source = "\"Hello,\nworld\n!\"";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 2);

        assert_eq!(scanner.tokens[0].token_type, TokenType::StringLit);
        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::StringValue(s) => assert_eq!(s, "Hello,\nworld\n!"),
            _ => panic!("Incorrect LiteralValue type"),
        }
        assert_eq!(scanner.tokens[1].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_number_literals() {
        let source = "123.45\n67.0\n8";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 4);

        assert_eq!(scanner.tokens[0].token_type, TokenType::Number);
        match scanner.tokens[0].literal {
            Some(LiteralValue::FValue(x)) => assert_eq!(x, 123.45),
            _ => panic!("Incorrect literal type"),
        }

        assert_eq!(scanner.tokens[1].token_type, TokenType::Number);
        match scanner.tokens[1].literal {
            Some(LiteralValue::FValue(x)) => assert_eq!(x, 67.0),
            _ => panic!("Incorrect literal type"),
        }

        assert_eq!(scanner.tokens[2].token_type, TokenType::Number);
        match scanner.tokens[2].literal {
            Some(LiteralValue::FValue(x)) => assert_eq!(x, 8.0),
            _ => panic!("Incorrect literal type"),
        }

        assert_eq!(scanner.tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_identifier() {
        let source = "varname = 6;";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 5);

        assert_eq!(scanner.tokens[0].token_type, TokenType::Identifier);
        assert_eq!(scanner.tokens[1].token_type, TokenType::Equal);
        assert_eq!(scanner.tokens[2].token_type, TokenType::Number);
        assert_eq!(scanner.tokens[3].token_type, TokenType::Semicolon);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_keywords() {
        let source = "var varname = 6.3;\nwhile true { print 123 };";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 13);

        assert_eq!(scanner.tokens[0].token_type, TokenType::Var);
        assert_eq!(scanner.tokens[1].token_type, TokenType::Identifier);
        assert_eq!(scanner.tokens[2].token_type, TokenType::Equal);
        assert_eq!(scanner.tokens[3].token_type, TokenType::Number);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(scanner.tokens[5].token_type, TokenType::While);
        assert_eq!(scanner.tokens[6].token_type, TokenType::True);
        assert_eq!(scanner.tokens[7].token_type, TokenType::LeftBrace);
        assert_eq!(scanner.tokens[8].token_type, TokenType::Print);
        assert_eq!(scanner.tokens[9].token_type, TokenType::Number);
        assert_eq!(scanner.tokens[10].token_type, TokenType::RightBrace);
        assert_eq!(scanner.tokens[11].token_type, TokenType::Semicolon);
        assert_eq!(scanner.tokens[12].token_type, TokenType::Eof);
    }
}
