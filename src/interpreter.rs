use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expression::{Expr, LiteralValue};
use crate::scanner::{Token, TokenType};
use crate::statement::Stmt;

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub specials: HashMap<String, LiteralValue>,
}

fn clock_impl(_args: &[LiteralValue]) -> LiteralValue {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Could not get system time.")
        .as_millis();

    LiteralValue::Number(now as f64 / 1000.0)
}

impl Interpreter {
    pub fn new() -> Self {
        let mut environment = Environment::new();

        environment.define(
            String::from("clock"),
            LiteralValue::OldCallable {
                name: String::from("clock"),
                arity: 0,
                fun: Rc::new(clock_impl),
            },
        );

        Self {
            environment: Rc::new(RefCell::new(environment)),
            specials: HashMap::new(),
        }
    }

    pub fn for_lambda(parent: Rc<RefCell<Environment>>) -> Self {
        let mut environment = Environment::new();
        environment.enclosing = Some(parent);

        Self {
            environment: Rc::new(RefCell::new(environment)),
            specials: HashMap::new(),
        }
    }

    fn for_closure(parent: Rc<RefCell<Environment>>) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().enclosing = Some(parent);

        Self {
            environment,
            specials: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralValue, String> {
        match expr {
            Expr::Assign { name, value } => {
                let new_value = self.evaluate(value)?;
                let success = self
                    .environment
                    .borrow_mut()
                    .assign(&name.lexeme, new_value.clone());

                if success {
                    Ok(new_value)
                } else {
                    Err(format!("Variable {} has not been declared.", name.lexeme))
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let expr_l = self.evaluate(left)?;
                let expr_r = self.evaluate(right)?;

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
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                let callee_literal = self.evaluate(callee)?;

                if let LiteralValue::OldCallable { name, arity, fun } = callee_literal {
                    let mut arg_list = vec![];
                    for argument in arguments.iter() {
                        arg_list.push(self.evaluate(argument)?);
                    }

                    if arguments.len() != arity {
                        Err(format!(
                            "Callable {name} expected {arity} arguments, got {}.",
                            arguments.len()
                        ))
                    } else {
                        let mut argument_values = vec![];
                        for argument in arguments {
                            let value = self.evaluate(argument)?;
                            argument_values.push(value);
                        }

                        Ok(fun(&argument_values))
                    }
                } else if let LiteralValue::Callable(callable) = callee_literal {
                    let mut arg_list = vec![];
                    for argument in arguments.iter() {
                        arg_list.push(self.evaluate(argument)?);
                    }
                    if arg_list.len() != callable.arity() {
                        Err(format!(
                            "Callable {} expected {} arguments, got {}",
                            callable.name(),
                            callable.arity(),
                            arg_list.len()
                        ))
                    } else {
                        Ok(callable.call(self, arg_list)?)
                    }
                } else {
                    Err(format!("{} is not callable", callee_literal.to_type()))
                }
            }
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Lambda {
                paren: _,
                arguments,
                body,
            } => {
                let arity = arguments.len();
                let arguments = arguments.clone();
                let body = body.clone();
                let environment = self.environment.clone();

                let fun_impl = move |args: &[LiteralValue]| {
                    let mut lambda_int = Interpreter::for_lambda(environment.clone());

                    for (i, arg) in args.iter().enumerate() {
                        lambda_int
                            .environment
                            .borrow_mut()
                            .define(arguments[i].lexeme.clone(), (*arg).clone())
                    }

                    for stmt in body.iter() {
                        lambda_int
                            .execute(stmt)
                            .unwrap_or_else(|_| panic!("Evaluating field failed"));
                        if let Some(value) = lambda_int.specials.get("return") {
                            return value.clone();
                        }
                    }

                    LiteralValue::Nil
                };

                Ok(LiteralValue::OldCallable {
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
                let left = self.evaluate(left)?;

                if operator.token_type == TokenType::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                }

                self.evaluate(right)
            }
            Expr::Unary { operator, right } => {
                let expr = self.evaluate(right)?;

                match (&expr, operator.token_type) {
                    (LiteralValue::Number(x), TokenType::Minus) => Ok(LiteralValue::Number(-x)),
                    (_, TokenType::Minus) => Err(format!(
                        "Minus operator not implemented for {}.",
                        expr.to_type()
                    )),
                    (value, TokenType::Bang) => Ok(LiteralValue::from_bool(!value.is_truthy())),
                    (_, token_type) => Err(format!("{token_type} is not a valid unary operator.")),
                }
            }
            Expr::Variable { name } => match self.environment.borrow().get(&name.lexeme) {
                Some(value) => Ok(value),
                None => Err(format!("Variable '{}' has not been declared.", name.lexeme)),
            },
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            self.execute(stmt)?
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements } => {
                let mut new_environment = Environment::new();
                new_environment.enclosing = Some(self.environment.clone());
                let old_environment = self.environment.clone();
                self.environment = Rc::new(RefCell::new(new_environment));
                let block_result = self.interpret(statements.iter().collect());
                self.environment = old_environment;
                block_result?
            }
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
            }
            Stmt::Function { name, params, body } => {
                let arity = params.len();

                let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                let body: Vec<Stmt> = body.iter().map(|b| (*b).clone()).collect();
                let name_clone = name.lexeme.clone();

                let parent_env = self.environment.clone();
                let fun_impl = move |args: &[LiteralValue]| {
                    let mut closure_int = Interpreter::for_closure(parent_env.clone());
                    for (i, arg) in args.iter().enumerate() {
                        closure_int
                            .environment
                            .borrow_mut()
                            .define(params[i].lexeme.clone(), (*arg).clone());
                    }

                    for item in &body {
                        closure_int
                            .execute(item)
                            .unwrap_or_else(|_| panic!("Evaluating failed inside {name_clone}."));

                        if let Some(value) = closure_int.specials.get("return") {
                            return value.clone();
                        }
                    }

                    LiteralValue::Nil
                };

                let callable = LiteralValue::OldCallable {
                    name: name.lexeme.clone(),
                    arity,
                    fun: Rc::new(fun_impl),
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), callable);
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let truth_value = self.evaluate(condition)?;
                if truth_value.is_truthy() {
                    self.execute(then_stmt)?
                } else if let Some(els) = else_stmt {
                    self.execute(els)?
                };
            }
            Stmt::Print { expression } => {
                let result = self.evaluate(expression)?;
                println!("{result}");
            }
            Stmt::Return { keyword: _, value } => {
                let value = if let Some(expr) = value {
                    self.evaluate(expr)?
                } else {
                    LiteralValue::Nil
                };
                self.specials.insert(String::from("return"), value);
            }
            Stmt::Var { name, initializer } => {
                let value = self.evaluate(initializer)?;
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
            }
            Stmt::While { condition, body } => {
                let mut flag = self.evaluate(condition)?;
                while flag.is_truthy() {
                    let statements: Vec<&Stmt> = vec![body.as_ref()];
                    self.interpret(statements)?;
                    flag = self.evaluate(condition)?;
                }
            }
        };
        Ok(())
    }
}
