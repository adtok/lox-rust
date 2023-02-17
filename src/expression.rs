use std::{cell::RefCell, rc::Rc};

use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::scanner::{self, Token, TokenType};
use crate::statement::Stmt;

#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
    Callable {
        name: String,
        arity: usize,
        fun: CallableFunction,
    },
}
pub type CallableFunction = Rc<dyn Fn(&[LiteralValue]) -> LiteralValue>;

impl LiteralValue {
    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            TokenType::Number => {
                let value = match token.literal {
                    Some(scanner::LiteralValue::FValue(x)) => x,
                    _ => panic!("Cannot be unwrapped as float"),
                };
                Self::Number(value)
            }
            TokenType::StringLit => {
                let value = match token.literal {
                    Some(scanner::LiteralValue::StringValue(s)) => s,
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
            LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => "Callable",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Number(x) => *x != 0.0f64,
            LiteralValue::StringValue(s) => !s.is_empty(),
            LiteralValue::True => true,
            LiteralValue::False => false,
            LiteralValue::Nil => false,
            LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => panic!("Cannot use a callable as a truthy value"),
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
            LiteralValue::Callable {
                name,
                arity,
                fun: _,
            } => format!("<fn {name}/{arity}>"),
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
            (
                LiteralValue::Callable {
                    name,
                    arity,
                    fun: _,
                },
                LiteralValue::Callable {
                    name: o_name,
                    arity: o_arity,
                    fun: _,
                },
            ) => name == o_name && arity == o_arity,
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
        arguments: Vec<Token>,
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

impl Expr {
    pub fn _evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<LiteralValue, String> {
        match self {
            Expr::Assign { name, value } => {
                let new_value = (*value)._evaluate(environment.clone())?;
                let success = environment
                    .borrow_mut()
                    .assign(&name.lexeme, new_value.clone());
                if success {
                    Ok(new_value)
                } else {
                    Err(format!("Variable {} has not been declared.", name.lexeme))
                }
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                let callable = (*callee)._evaluate(environment.clone())?;

                match callable {
                    LiteralValue::Callable { name, arity, fun } => {
                        let mut arg_list = vec![];
                        for argument in arguments.iter() {
                            arg_list.push(argument._evaluate(environment.clone())?);
                        }

                        if arguments.len() != arity {
                            return Err(format!(
                                "Callable {} expected {} arguments, got {}.",
                                name,
                                arity,
                                arguments.len(),
                            ));
                        }

                        let mut argument_values = vec![];
                        for argument in arguments {
                            let value = argument._evaluate(environment.clone())?;
                            argument_values.push(value);
                        }

                        Ok(fun(&argument_values))
                    }
                    other => Err(format!("{} is not callable.", other.to_type())),
                }
            }
            Expr::Lambda {
                paren: _,
                arguments,
                body,
            } => {
                let arity = arguments.len();
                let environment = environment;
                let arguments = arguments.clone();
                let body = body.clone();
                // let body: Vec<Stmt> = body.iter().map(|b| (*b).clone()).collect();

                let fun_impl = move |args: &[LiteralValue]| {
                    let mut lambda_int = Interpreter::for_lambda(environment.clone());

                    for (i, arg) in args.iter().enumerate() {
                        lambda_int
                            .environment
                            .borrow_mut()
                            .define(arguments[i].lexeme.clone(), (*arg).clone());
                    }

                    for stmt in body.iter() {
                        lambda_int
                            .interpret(vec![stmt])
                            .unwrap_or_else(|_| panic!("Evaluating field failed."));

                        if let Some(value) = lambda_int.specials.get("return") {
                            return value.clone();
                        }
                    }

                    LiteralValue::Nil
                };

                Ok(LiteralValue::Callable {
                    name: String::from("lambda"),
                    arity,
                    fun: Rc::new(fun_impl),
                })
            }
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left._evaluate(environment.clone())?;

                if operator.token_type == TokenType::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                }

                right._evaluate(environment)
            }
            Expr::Grouping { expression } => expression._evaluate(environment),
            Expr::Unary { operator, right } => {
                let expr = right._evaluate(environment)?;

                match (&expr, operator.token_type) {
                    (LiteralValue::Number(x), TokenType::Minus) => Ok(LiteralValue::Number(-x)),
                    (_, TokenType::Minus) => Err(format!(
                        "Minus operator not implemented for {}.",
                        expr.to_type()
                    )),
                    (value, TokenType::Bang) => Ok(LiteralValue::from_bool(!value.is_truthy())),
                    (_, token_type) => Err(format!("{token_type} is not a valid unary operator")),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let expr_l = left._evaluate(environment.clone())?;
                let expr_r = right._evaluate(environment)?;

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
                        Err(format!("{tt} is not supported for String and Number"))
                    }
                    (LiteralValue::StringValue(_), tt, LiteralValue::Number(_)) => {
                        Err(format!("{tt} is not supported for String and Number"))
                    }
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Plus,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::StringValue(format!("{s1}{s2}"))),
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
                    (x, tt, y) => Err(format!("{tt} is not supported for {x:?} and {y:?}")),
                }
            }
            Expr::Variable { name } => match environment.borrow().get(&name.lexeme) {
                Some(value) => Ok(value),
                None => Err(format!("Variable {} has not been declared.", name.lexeme)),
            },
        }
    }
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
                arguments,
                body: _,
            } => format!("anon/{}", arguments.len()),
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
