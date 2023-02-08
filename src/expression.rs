use crate::scanner::{Token, TokenType};

pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(s) => s.clone(),
            LiteralValue::True => String::from("true"),
            LiteralValue::False => String::from("false"),
            LiteralValue::Nil => String::from("nil"),
        }
    }
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        expression: Box<Expr>,
    },
}

impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Binary {
                left,
                right,
                operator,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string(),
            ),
            Expr::Grouping { expression } => format!("(group {})", (*expression).to_string()),
            Expr::Literal { value } => format!("{}", value.to_string()),
            Expr::Unary {
                operator,
                expression,
            } => {
                let operator_str = operator.lexeme.clone();
                let expression_str = (*expression).to_string();
                format!("({} {})", operator_str, expression_str)
            }
        }
    }

    pub fn print(&self) {
        println!("{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_print_ast() {
        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: String::from("-"),
            literal: None,
            line: 0,
        };
        let onetwothree = Expr::Literal {
            value: LiteralValue::Number(123.0),
        };
        let group = Expr::Grouping {
            expression: Box::from(Expr::Literal {
                value: LiteralValue::Number(45.67),
            }),
        };
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: String::from("*"),
            literal: None,
            line: 0,
        };
        let ast = Expr::Binary {
            left: Box::from(Expr::Unary {
                operator: minus_token,
                expression: Box::from(onetwothree),
            }),
            operator: multi,
            right: Box::from(group),
        };
        let result = ast.to_string();
        assert_eq!(result, "(* (- 123) (group 45.67))")
    }
}
