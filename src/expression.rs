use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    scanner::{self, Token, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
}

impl LiteralValue {
    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            TokenType::Number => {
                let value = match token.literal {
                    Some(scanner::LiteralValue::FValue(x)) => x as f64,
                    _ => panic!("Cannot be unwrapped as float"),
                };
                Self::Number(value)
            }
            TokenType::StringLit => {
                let value = match token.literal {
                    Some(scanner::LiteralValue::StringValue(s)) => s.clone(),
                    _ => panic!("Cannot be unwrapped as String"),
                };
                Self::StringValue(value)
            }
            TokenType::False => Self::False,
            TokenType::Nil => Self::Nil,
            TokenType::True => Self::True,
            _ => panic!("Could not create LiteralValue from {:?}", token),
        }
    }

    pub fn from_bool(boolean: bool) -> Self {
        if boolean {
            Self::True
        } else {
            Self::False
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(s) => s.clone(),
            LiteralValue::True => String::from("true"),
            LiteralValue::False => String::from("false"),
            LiteralValue::Nil => String::from("nil"),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "Number",
            LiteralValue::StringValue(_) => "String",
            LiteralValue::True => "Boolean",
            LiteralValue::False => "Boolean",
            LiteralValue::Nil => "nil",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Number(x) => *x != 0.0 as f64,
            LiteralValue::StringValue(s) => s.len() > 0,
            LiteralValue::True => true,
            LiteralValue::False => false,
            LiteralValue::Nil => false,
        }
    }
}

#[derive(Debug)]
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
    Grouping {
        expression: Box<Expr>,
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

impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Assign { name, value } => {
                format!("({name:?} = {}", value.to_string())
            }
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
            Expr::Logical {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string()
            ),
            Expr::Unary {
                operator,
                right: expression,
            } => {
                let operator_str = operator.lexeme.clone();
                let expression_str = (*expression).to_string();
                format!("({} {})", operator_str, expression_str)
            }
            Expr::Variable { name } => format!("(var {})", name.lexeme),
        }
    }

    pub fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<LiteralValue, String> {
        match self {
            Expr::Assign { name, value } => {
                let new_value = (*value).evaluate(environment.clone())?;
                let success = environment
                    .borrow_mut()
                    .assign(&name.lexeme, new_value.clone());
                if success {
                    Ok(new_value)
                } else {
                    Err(format!("Variable {} has not been declared.", name.lexeme))
                }
            }
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(environment.clone())?;

                if operator.token_type == TokenType::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else {
                    if !left.is_truthy() {
                        return Ok(left);
                    }
                }

                right.evaluate(environment.clone())
            }
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let expr = right.evaluate(environment)?;

                match (&expr, operator.token_type) {
                    (LiteralValue::Number(x), TokenType::Minus) => Ok(LiteralValue::Number(-x)),
                    (_, TokenType::Minus) => Err(format!(
                        "Minus operator not implemented for {}.",
                        expr.to_type()
                    )),
                    (value, TokenType::Bang) => Ok(LiteralValue::from_bool(!value.is_truthy())),
                    (_, token_type) => Err(format!("{} is not a valid unary operator", token_type)),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let expr_l = left.evaluate(environment.clone())?;
                let expr_r = right.evaluate(environment.clone())?;

                match (&expr_l, operator.token_type, &expr_r) {
                    (LiteralValue::Number(x), TokenType::Plus, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x + y))
                    }
                    (LiteralValue::Number(x), TokenType::Minus, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x - y))
                    }
                    (LiteralValue::Number(x), TokenType::Star, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x * y))
                    }
                    (LiteralValue::Number(x), TokenType::Slash, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x / y))
                    }
                    (LiteralValue::Number(x), TokenType::Greater, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from_bool(x > y))
                    }
                    (LiteralValue::Number(x), TokenType::GreaterEqual, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from_bool(x >= y))
                    }
                    (LiteralValue::Number(x), TokenType::Less, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from_bool(x < y))
                    }
                    (LiteralValue::Number(x), TokenType::LessEqual, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from_bool(x <= y))
                    }
                    (LiteralValue::Number(_), tt, LiteralValue::StringValue(_)) => {
                        Err(format!("{} is not supported for String and Number", tt))
                    }
                    (LiteralValue::StringValue(_), tt, LiteralValue::Number(_)) => {
                        Err(format!("{} is not supported for String and Number", tt))
                    }
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Plus,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::StringValue(format!("{}{}", s1, s2))),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Greater,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from_bool(s1 > s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::GreaterEqual,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from_bool(s1 >= s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Less,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from_bool(s1 < s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::LessEqual,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from_bool(s1 <= s2)),
                    (x, TokenType::BangEqual, y) => Ok(LiteralValue::from_bool(x != y)),
                    (x, TokenType::EqualEqual, y) => Ok(LiteralValue::from_bool(x == y)),
                    (x, tt, y) => Err(format!("{} is not supported for {:?} and {:?}", tt, x, y)),
                }
            }
            Expr::Variable { name } => match environment.borrow().get(&name.lexeme) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Variable {} has not been declared.", name.lexeme)),
            },
        }
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
}
