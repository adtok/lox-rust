use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expression::{CallableFunction, Expr, LiteralValue, LoxCallable};
use crate::scanner::{Token, TokenType};
use crate::statement::Stmt;

pub struct Interpreter {
    pub environment: Environment,
    pub specials: HashMap<String, LiteralValue>,
    globals: Environment,
    locals: HashMap<Expr, usize>,
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
            LiteralValue::Callable(LoxCallable::NativeFunction {
                name: String::from("clock"),
                arity: 0,
                fun: Rc::new(clock_impl),
            }),
        );

        Self {
            environment: environment,
            globals: Environment::new(),
            specials: HashMap::new(),
            locals: HashMap::new(),
        }
    }

    pub fn for_lambda(parent: Rc<RefCell<Environment>>) -> Self {
        let mut environment = Environment::new();
        environment.enclosing = Some(parent);

        Self {
            environment: environment,
            globals: Environment::new(),
            specials: HashMap::new(),
            locals: HashMap::new(),
        }
    }

    pub fn for_closure(parent: Environment) -> Self {
        let environment = Environment::new();
        environment.borrow_mut().enclosing = Some(parent);

        Self {
            environment,
            globals: Environment::new(),
            specials: HashMap::new(),
            locals: HashMap::new(),
        }
    }

    pub fn globals_clone(&mut self) -> Environment {
        self.globals.clone()
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralValue, String> {
        match expr {
            Expr::Assign { name, value } => {
                let new_value = self.evaluate(value)?;

                // let success = match self.locals.get(expr) {
                //     Some(distance) => {
                //         println!("distance={distance}");
                //         (*self.environment).borrow_mut().assign_at(
                //             *distance,
                //             &name.lexeme,
                //             new_value.clone(),
                //         )
                //     }
                //     None => {
                //         println!("sad");
                //         self.globals.assign(&name.lexeme, new_value.clone())
                //     }
                // };

                // if success {
                //     Ok(new_value)
                // } else {
                //     Err(format!(
                //         "Variable '{}' has not been declared in this scope..",
                //         name.lexeme
                //     ))
                // }

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
                let literal_value = self.evaluate(callee)?;

                if let LiteralValue::Callable(callable) = literal_value {
                    callable.call(self, arguments)
                } else {
                    Err("Expected callable".to_string())
                }
            }
            // Expr::Call {
            //     callee,
            //     paren: _,
            //     arguments,
            // } => {
            //     let literal_value = self.evaluate(callee)?;

            //     println!("callee={callee}");
            //     // let num_args = arguments.len();

            //     // let mut arg_list = vec![];

            //     // for argument in arguments.iter() {
            //     //     arg_list.push(self.evaluate(argument)?);
            //     // }

            //     println!("callable={literal_value}");
            //     if let LiteralValue::Callable(callable) = literal_value {
            //         match callable {
            //             LoxCallable::LoxFunction {
            //                 name,
            //                 arity,
            //                 mut parent_env,
            //                 params,
            //                 body,
            //             } => {
            //                 let num_args = arguments.len();
            //                 if num_args != arity {
            //                     return Err(format!("Expected {arity} arguments got {num_args} when calling {name}."));
            //                 }

            //                 println!("{:?}", self.environment.borrow().values);
            //                 println!("{:?}", parent_env.values);

            //                 let mut arg_list = vec![];

            //                 for argument in arguments.iter() {
            //                     arg_list.push(self.evaluate(argument)?);
            //                 }
            //                 // let mut environment = self.globals.clone();
            //                 for (i, val) in arg_list.iter().enumerate() {
            //                     self.environment
            //                         .borrow_mut()
            //                         .define(params[i].lexeme.clone(), val.clone());
            //                 }

            //                 let mut interpreter =
            //                     Interpreter::for_closure(Rc::new(RefCell::new(parent_env.clone())));

            //                 for i in 0..body.len() {
            //                     self.execute(&body[i].clone())?;

            //                     if let Some(value) = self.specials.get("return") {
            //                         return Ok(value.clone());
            //                     }
            //                 }

            //                 Ok(LiteralValue::Nil)
            //             }
            //             LoxCallable::NativeFunction { name, arity, fun } => {
            //                 let num_args = arguments.len();
            //                 if num_args != arity {
            //                     return Err(format!("Expected {arity} arguments got {num_args} when calling {name}."));
            //                 }

            //                 let mut arg_list = vec![];

            //                 for argument in arguments.iter() {
            //                     arg_list.push(self.evaluate(argument)?);
            //                 }
            //                 Ok(fun(&arg_list))
            //             }
            //         }
            //     } else {
            //         Err(format!("{} is not callable", literal_value.to_type()))
            //     }
            // }
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Lambda {
                paren: _,
                arguments,
                body,
            } => {
                let arity = arguments.len();
                let arguments = arguments.clone();
                let body: Vec<Box<Stmt>> = body.iter().map(|s| Box::new(s.clone())).collect();
                let closure = self.environment.borrow().clone();

                let lox_callable = LoxCallable::LoxFunction {
                    name: String::from("lambda"),
                    arity,
                    closure,
                    body,
                    params: arguments,
                    // fun: Rc::new(fun_impl),
                };
                Ok(LiteralValue::Callable(lox_callable))
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
            // Expr::Variable { name } => match self.look_up_variable(name.clone(), expr.clone()) {
            //     Some(value) => Ok(value),
            //     None => Err(format!("Variable '{}' has not been declared.", name.lexeme)),
            // },
            Expr::Variable { name } => match (*self.environment).borrow().get(&name.lexeme) {
                Some(value) => Ok(value),
                None => Err(format!("Variable '{}' has not been declared.", name.lexeme)),
            },
        }
    }

    fn look_up_variable(&mut self, name: Token, expression: Expr) -> Option<LiteralValue> {
        // println!("luv");
        // println!("{:?}", self.locals);
        if let Some(distance) = self.locals.get(&expression) {
            (*self.environment).borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.get(&name.lexeme)
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements } => {
                let new_environment = Environment::new();
                let block_result = self.execute_block(statements.iter().collect(), new_environment);
                block_result?
            }
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
            }
            Stmt::Function { name, params, body } => {
                let arity = params.len();

                let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                let body: Vec<Box<Stmt>> = body.iter().map(|b| Box::new(b.clone())).collect();
                let name = name.lexeme.clone();

                let callable = LiteralValue::Callable(LoxCallable::LoxFunction {
                    name: name.clone(),
                    arity,
                    closure: self.environment.borrow_mut().clone(),
                    params,
                    body,
                });

                self.environment.borrow_mut().define(name.clone(), callable);
                // println!("{:?}", self.environment.borrow().values);
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let truth_value = self.evaluate(condition)?;
                if truth_value.is_truthy() {
                    self.interpret(vec![then_stmt])?
                } else if let Some(els) = else_stmt {
                    self.interpret(vec![els])?
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

    pub fn execute_block(
        &mut self,
        statements: Vec<&Stmt>,
        mut environment: Environment,
    ) -> Result<(), String> {
        environment.enclosing = Some(self.environment.clone());
        let old_environment = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        let result = self.interpret(statements);
        self.environment = old_environment;

        result
    }

    pub fn resolve(&mut self, expression: &Expr, steps: usize) -> Result<(), String> {
        // println!("In resolve");
        self.locals.insert(expression.clone(), steps);
        // println!("{} {:?}", self.locals.len(), self.locals);
        Ok(())
    }
}
