use crate::scanner::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn expression(&mut self) {
        self.equality()
    }

    fn equality(&mut self) {
        let mut expr = self.comparison();
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            if self.peek().token_type == token_type {
                self.advance()
                true
            } else {
                false
            }
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        types.iter().any(|t| self.match_token(*t))
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }

        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {}
}
