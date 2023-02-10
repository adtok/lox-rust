use crate::{
    expression::{Expr, LiteralValue},
    scanner::{Token, TokenType},
    statement::Stmt,
};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        let mut errors = vec![];

        while !self.is_at_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(msg) => {
                    errors.push(msg);
                    self.synchronize();
                }
            }
        }

        if errors.len() == 0 {
            Ok(stmts)
        } else {
            Err(errors.join("\n"))
        }
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.check(TokenType::Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Expression { expression: value })
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;

        if self.check(TokenType::Equal) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::from(value),
                }),
                _ => Err(format!("{:?}: Invalid Assignment target", equals)),
            }
        } else {
            Ok(expr)
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;

        let initializer;
        if self.check(TokenType::Equal) {
            initializer = self.expression()?;
        } else {
            initializer = Expr::Literal {
                value: LiteralValue::Nil,
            };
        }

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.check(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::from(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();

        let result = match token.token_type {
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')'")?;
                Expr::Grouping {
                    expression: Box::from(expr),
                }
            }
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::StringLit => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::from_token(token),
                }
            }
            TokenType::Identifier => {
                self.advance();
                Expr::Variable {
                    name: self.previous(),
                }
            }
            _ => return Err(String::from("Expected an expression.")),
        };

        Ok(result)
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(*t) {
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, String> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            Ok(self.previous())
        } else {
            Err(String::from(message))
        }
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else if self.peek().token_type == token_type {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{LiteralValue, Scanner, Token, TokenType};

    #[test]
    fn test_addition() {
        let one = Token {
            token_type: TokenType::Number,
            lexeme: String::from("1"),
            literal: Some(LiteralValue::IntValue(1)),
            line: 0,
        };
        let plus = Token {
            token_type: TokenType::Plus,
            lexeme: String::from("+".to_string()),
            literal: None,
            line: 0,
        };
        let two = Token {
            token_type: TokenType::Number,
            lexeme: String::from("2"),
            literal: Some(LiteralValue::IntValue(2)),
            line: 0,
        };
        let semicolon = Token {
            token_type: TokenType::Semicolon,
            lexeme: String::from(";"),
            literal: None,
            line: 0,
        };
        let eof = Token {
            token_type: TokenType::Eof,
            lexeme: String::from(""),
            literal: None,
            line: 0,
        };

        let tokens = vec![one, plus, two, semicolon, eof];
        let mut parser = Parser::new(tokens);

        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(+ 1 2)");
    }

    #[test]
    fn test_comparison() {
        let source = "1 + 1 == 5 + 7;";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(== (+ 1 1) (+ 5 7))");
    }

    #[test]
    fn test_comparison_with_parens() {
        let source = "1 >= (3 + 4);";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();

        assert_eq!(string_expr, "(>= 1 (group (+ 3 4)))");
    }
}
