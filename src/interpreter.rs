use crate::callable::LoxCallable;
use crate::environment::Environment;
use crate::expression::{Expr, LiteralValue};
use crate::scanner::{Token, TokenType};
use crate::statement::Stmt;

pub struct Interpreter {
    pub lambda_counter: usize,
    pub globals: Environment,
    pub environment: Environment,
    pub return_value: Option<LiteralValue>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();

        let clock_token = Token::global("clock");
        globals.define(
            clock_token.clone(),
            LiteralValue::Callable(LoxCallable::NativeFunction {
                name: clock_token.lexeme,
                arity: 0,
                fun: |_, _| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .expect("Could not get system time.")
                        .as_millis();

                    Ok(LiteralValue::Number(now as f64 / 1000.0))
                },
            }),
        );

        Self {
            lambda_counter: 0,
            globals: Environment::new(),
            environment: globals,
            return_value: None,
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralValue, String> {
        match expr {
            Expr::Assign { name, value } => {
                let new_value = self.evaluate(value)?;

                self.environment.assign(name.clone(), &new_value)?;

                Ok(new_value)
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
            } => self.call(callee, arguments),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Lambda {
                paren,
                params,
                body,
            } => {
                let lambda_sym = self.lambda_name();
                let name = Token {
                    token_type: TokenType::Identifier,
                    lexeme: lambda_sym,
                    literal: None,
                    line: paren.line,
                };

                let maybe_err = self.execute(&Stmt::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                });

                match maybe_err {
                    Ok(_) => self.evaluate(&Expr::Variable { name }),
                    Err(msg) => Err(msg),
                }
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
            Expr::Variable { name } => match self.environment.get(&name.lexeme) {
                Some(value) => Ok(value),
                None => Err(format!("Variable '{}' has not been declared.", name.lexeme)),
            },
        }
    }

    fn lambda_name(&mut self) -> String {
        let res = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;
        res
    }

    fn call(&mut self, callee_expr: &Expr, arg_exprs: &[Expr]) -> Result<LiteralValue, String> {
        let callee = self.evaluate(callee_expr)?;

        if let Some(callable) = callee.as_callable() {
            let maybe_args: Result<Vec<_>, _> =
                arg_exprs.iter().map(|arg| self.evaluate(arg)).collect();
            match maybe_args {
                Ok(args) => {
                    if args.len() != callable.arity() {
                        Err(format!(
                            "Call for '{}' expected {} args, got {}",
                            callable.name(),
                            callable.arity(),
                            args.len()
                        ))
                    } else {
                        callable.call(self, &args)
                    }
                }
                Err(msg) => Err(msg),
            }
        } else {
            Err(format!("Attempted to call non-callable '{callee}'"))
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            self.execute(stmt)?
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        if self.return_value.is_some() {
            return Ok(());
        }
        match stmt {
            Stmt::Block { statements } => {
                self.environment = Environment::with_enclosing(self.environment.clone());
                for statement in statements.iter() {
                    self.execute(statement)?;
                }

                if let Some(enclosing) = self.environment.enclosing.clone() {
                    self.environment = *enclosing
                } else {
                    panic!("impossible to reach here");
                }
            }
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
            }
            Stmt::Function { name, params, body } => {
                let callable = LiteralValue::Callable(LoxCallable::LoxFunction {
                    name: name.lexeme.clone(),
                    parameters: params.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                });

                self.environment.define(name.clone(), callable);
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
                }
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
                self.return_value = Some(value);
            }
            Stmt::Var { name, initializer } => {
                let value = self.evaluate(initializer)?;
                self.environment.define(name.clone(), value);
            }
            Stmt::While { condition, body } => {
                let mut flag = self.evaluate(condition)?;
                while flag.is_truthy() {
                    self.execute(body)?;
                    flag = self.evaluate(condition)?;
                }
            }
        };
        Ok(())
    }
}
