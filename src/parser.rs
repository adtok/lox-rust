use crate::{
    expression::{Expr, LiteralValue},
    scanner::{Token, TokenType},
    statement::Stmt,
};

#[derive(Debug)]
enum FunctionKind {
    Function,
}

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
            let line = self.peek().line;
            let stmt = self.declaration();
            match stmt {
                Ok(stmt) => stmts.push(stmt),
                Err(msg) => {
                    errors.push(format!("Line {line}: {msg}"));
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(stmts)
        } else {
            Err(errors.join("\n"))
        }
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_tokens(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_tokens(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_tokens(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_tokens(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_tokens(&[TokenType::LeftBrace]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;

        let initializer = if self.match_tokens(&[TokenType::Semicolon]) {
            None
        } else if self.match_tokens(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.match_tokens(&[TokenType::Semicolon]) {
            Expr::Literal {
                value: LiteralValue::True,
            }
        } else {
            self.expression()?
        };
        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(
            TokenType::RightParen,
            "Expected ')' after for loop clauses.",
        )?;

        let mut body = self.statement()?;

        if let Some(increment_stmt) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: increment_stmt,
                    },
                ],
            }
        };

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer_stmt) = initializer {
            body = Stmt::Block {
                statements: vec![initializer_stmt, body],
            }
        };

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after 'if'.")?;

        let then_stmt = Box::new(self.statement()?);
        let else_stmt = if self.match_tokens(&[TokenType::Else]) {
            let stmt = self.statement()?;
            Some(Box::new(stmt))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_stmt,
            else_stmt,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let keyword = self.previous();

        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Expression { expression: value })
    }

    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements: Vec<Stmt> = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let declaration = self.declaration()?;
            statements.push(declaration);
        }

        match self.consume(TokenType::RightBrace, "Expected '}' after a block") {
            Ok(_) => Ok(Stmt::Block { statements }),
            Err(msg) => Err(msg),
        }
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_tokens(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.expression()?;

            match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::from(value),
                }),
                _ => Err(format!("{equals:?}: Invalid Assignment target")),
            }
        } else {
            Ok(expr)
        }
    }

    fn lambda_expression(&mut self) -> Result<Expr, String> {
        let paren = self.consume(TokenType::LeftParen, "Expected '(' after lambda function.")?;
        let mut params = vec![];

        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(String::from(
                        "Can't have more than 255 arguments in a lambda function.",
                    ));
                }

                let param = self.consume(TokenType::Identifier, "Expected parameter name.")?;
                params.push(param);

                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            "Expected ')' after lambda function parameters.",
        )?;

        self.consume(
            TokenType::LeftBrace,
            "Expected '{' after lambda function declaration.",
        )?;

        let body = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => panic!("Block statement parsed something that was not a block."),
        };

        Ok(Expr::Lambda {
            paren,
            arguments: params,
            body,
        })
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_tokens(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_tokens(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn fun_declaration(&mut self, kind: FunctionKind) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, &format!("Expected {kind:?} name."))?;

        self.consume(
            TokenType::LeftParen,
            &format!("Expected '(' after {kind:?} name."),
        )?;

        let mut params = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(String::from("Can't have more than 255 parameters."));
                }

                params.push(self.consume(TokenType::Identifier, "Expected parameter name.")?);

                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {kind:?} body."),
        )?;
        let body = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => panic!("Found something other than a block"),
        };

        let s = Stmt::Function { name, params, body };

        Ok(s)
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;

        let initializer = if self.match_tokens(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal {
                value: LiteralValue::Nil,
            }
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after a 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Fun]) {
            self.fun_declaration(FunctionKind::Function)
        } else if self.match_tokens(&[TokenType::Var]) {
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
            self.call()
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments: Vec<Expr> = vec![];

        if !self.check(TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);

                if arguments.len() >= 255 {
                    // Change to handle gracefully if ever implemented
                    return Err(String::from(
                        "Functions cannot have more than 255 arguments",
                    ));
                }

                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
            paren,
        })
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_tokens(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
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
            TokenType::Fun => {
                self.advance();
                self.lambda_expression()?
            }
            other => return Err(format!("Expected an expression, got {other:?}.")),
        };

        Ok(result)
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(*t) {
                self.advance();
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
            Err(String::from(message)) // TODO: Adjust parameters to take String
        }
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
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
            literal: Some(LiteralValue::FValue(1.0)),
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
            literal: Some(LiteralValue::FValue(2.0)),
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
