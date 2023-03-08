use std::rc::Rc;

use crate::callable::LoxCallable;
use crate::scanner::{Token, TokenLiteral, TokenType};
use crate::statement::Stmt;

#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
    Callable(LoxCallable),
}
pub type CallableFunction = Rc<dyn Fn(&[LiteralValue]) -> LiteralValue>;

impl LiteralValue {
    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            TokenType::Number => {
                let value = match token.literal {
                    Some(TokenLiteral::FValue(x)) => x,
                    _ => panic!("Cannot be unwrapped as float"),
                };
                Self::Number(value)
            }
            TokenType::StringLit => {
                let value = match token.literal {
                    Some(TokenLiteral::StringValue(s)) => s,
                    _ => panic!("Cannot be unwrapped as String"),
                };
                Self::StringValue(value)
            }
            TokenType::False => Self::False,
            TokenType::Nil => Self::Nil,
            TokenType::True => Self::True,
            _ => panic!("Could not create LiteralValue from {token:?}"),
        }
    }

    pub fn from_bool(boolean: bool) -> Self {
        if boolean {
            Self::True
        } else {
            Self::False
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "Number",
            LiteralValue::StringValue(_) => "String",
            LiteralValue::True => "Boolean",
            LiteralValue::False => "Boolean",
            LiteralValue::Nil => "nil",
            LiteralValue::Callable(_) => "Callable",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Number(x) => *x != 0.0f64,
            LiteralValue::StringValue(s) => !s.is_empty(),
            LiteralValue::True => true,
            LiteralValue::False => false,
            LiteralValue::Nil => false,
            LiteralValue::Callable(_) => panic!("Cannot use callable as truthy value"),
        }
    }

    pub fn as_callable(&self) -> Option<LoxCallable> {
        match self {
            LiteralValue::Callable(callable) => Some(callable.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(s) => s.clone(),
            LiteralValue::True => String::from("true"),
            LiteralValue::False => String::from("false"),
            LiteralValue::Nil => String::from("nil"),
            LiteralValue::Callable(callable) => callable.to_string(),
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Debug for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LiteralValue::Number(x), LiteralValue::Number(y)) => x == y,
            (LiteralValue::StringValue(s1), LiteralValue::StringValue(s2)) => s1 == s2,
            (LiteralValue::True, LiteralValue::True) => true,
            (LiteralValue::False, LiteralValue::False) => true,
            (LiteralValue::Nil, LiteralValue::Nil) => true,
            (LiteralValue::Callable(c1), LiteralValue::Callable(c2)) => {
                c1.name() == c2.name() && c1.arity() == c2.arity()
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Lambda {
        paren: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Literal {
        value: LiteralValue,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Expr::Assign { name, value } => {
                format!("({name:?} = {value}")
            }
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let op = operator.lexeme.clone();
                format!("({op} {left} {right})")
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => format!("({callee} {arguments:?})"),
            Expr::Grouping { expression } => format!("(group {expression})"),
            Expr::Lambda {
                paren: _,
                params,
                body: _,
            } => format!("anon/{}", params.len()),
            Expr::Literal { value } => format!("{value}"),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let op = operator.lexeme.clone();
                format!("({op} {left} {right})")
            }
            Expr::Unary {
                operator,
                right: expression,
            } => {
                let operator_str = operator.lexeme.clone();
                // let expression_str = (*expression).to_string();
                format!("({operator_str} {expression})")
            }
            Expr::Variable { name } => format!("(var {})", name.lexeme),
        };
        write!(f, "{s}")
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
                right: Box::from(onetwothree),
            }),
            operator: multi,
            right: Box::from(group),
        };
        let result = ast.to_string();
        assert_eq!(result, "(* (- 123) (group 45.67))")
    }

    #[test]
    fn expr_is_truthy() {
        // Numbers
        let truthy_number1 = LiteralValue::Number(-1.2);
        let truthy_number2 = LiteralValue::Number(0.1);
        let truthy_number3 = LiteralValue::Number(100.1);
        let falsy_number = LiteralValue::Number(0.0);

        assert!(truthy_number1.is_truthy());
        assert!(truthy_number2.is_truthy());
        assert!(truthy_number3.is_truthy());
        assert!(!falsy_number.is_truthy());

        // Strings
        let truthy_string = LiteralValue::StringValue("False".to_string());
        let falsy_string = LiteralValue::StringValue("".to_string());

        assert!(truthy_string.is_truthy());
        assert!(!falsy_string.is_truthy());

        // True, False, Nil
        assert!(LiteralValue::True.is_truthy());
        assert!(!LiteralValue::False.is_truthy());
        assert!(!LiteralValue::Nil.is_truthy());
    }

    #[test]
    fn logical_expr() {
        assert!(true);
    }
}
