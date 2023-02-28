use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::scanner::{self, Token, TokenType};
use crate::statement::Stmt;

#[derive(Clone)]
pub enum LoxCallable {
    LoxFunction {
        name: String,
        arity: usize,
        closure: Environment,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    NativeFunction {
        name: String,
        arity: usize,
        fun: CallableFunction,
    },
}

impl LoxCallable {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Expr>,
    ) -> Result<LiteralValue, String> {
        match self {
            LoxCallable::LoxFunction {
                name,
                arity,
                closure,
                params,
                body,
            } => {
                let num_args = arguments.len();
                if num_args != *arity {
                    return Err(format!("{name} expects {arity} args, received {num_args}."));
                }
                // let mut environment = &interpreter.environment.borrow_mut().clone();
                let mut args = vec![];
                for argument in arguments.iter() {
                    args.push(interpreter.evaluate(argument)?);
                }
                // println!("{:?}", environment.values);
                let mut interpreter = Interpreter::for_closure(interpreter.environment.clone());
                for i in 0..params.len() {
                    interpreter
                        .environment
                        .borrow_mut()
                        .define(params[i].lexeme.clone(), args[i].clone());
                }
                // interpreter
                //     .execute_block(body.iter().map(|s| s.as_ref()).collect(), environment)?;

                for statement in body.iter() {
                    interpreter.execute(statement)?;

                    if let Some(val) = interpreter.specials.get("return") {
                        return Ok(val.clone());
                    }
                }

                Ok(LiteralValue::Nil)
            }
            LoxCallable::NativeFunction { name, arity, fun } => {
                let num_args = arguments.len();
                if num_args != *arity {
                    return Err(format!("{name} expects {arity} args, received {num_args}."));
                }
                let mut args = vec![];
                for argument in arguments.iter() {
                    args.push(interpreter.evaluate(argument)?);
                }
                Ok(fun(&args))
            }
        }
    }
}

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
            LiteralValue::Callable(_) => panic!("Cannot use a callable as a truthy value"),
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
            LiteralValue::Callable(func) => match func {
                LoxCallable::LoxFunction {
                    name,
                    arity,
                    closure: _,
                    params: _,
                    body: _,
                }
                | LoxCallable::NativeFunction {
                    name,
                    arity,
                    fun: _,
                } => format!("<fn {name}/{arity}>"),
            },
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
                LiteralValue::Callable(LoxCallable::LoxFunction {
                    name,
                    arity,
                    closure: _,
                    params: _,
                    body: _,
                }),
                LiteralValue::Callable(LoxCallable::LoxFunction {
                    name: o_name,
                    arity: o_arity,
                    closure: _,
                    params: _,
                    body: _,
                }),
            ) => name == o_name && arity == o_arity,
            (
                LiteralValue::Callable(LoxCallable::NativeFunction {
                    name,
                    arity,
                    fun: _,
                }),
                LiteralValue::Callable(LoxCallable::NativeFunction {
                    name: o_name,
                    arity: o_arity,
                    fun: _,
                }),
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

impl Expr {}

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

impl std::hash::Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // std::ptr::hash(self, state);
        match self {
            Expr::Assign { name, value: _ } => {
                name.lexeme.hash(state);
                // value.hash(state);
            }
            Expr::Call {
                callee,
                paren: _,
                arguments: _,
            } => {
                (*callee).hash(state);
            }
            Expr::Variable { name } => {
                name.lexeme.hash(state);
                // println!("hash is {}", state.finish());
            }
            _ => panic!("Hash not implemented for this one."),
        }
        // println!("hash for {self:?} is {}", state.finish());
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        let ptr = std::ptr::addr_of!(self);
        let ptr2 = std::ptr::addr_of!(other);
        ptr == ptr2
    }
}

impl Eq for Expr {}

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
