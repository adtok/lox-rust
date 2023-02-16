use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expression::LiteralValue;
use crate::scanner::Token;
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
            LiteralValue::Callable {
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

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
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
                    expression.evaluate(self.environment.clone())?;
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
                            closure_int.interpret(vec![item]).unwrap_or_else(|_| {
                                panic!("Evaluating failed inside {name_clone}.")
                            });

                            if let Some(value) = closure_int.specials.get("return") {
                                return value.clone();
                            }
                        }

                        LiteralValue::Nil
                    };

                    let callable = LiteralValue::Callable {
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
                    let truth_value = condition.evaluate(self.environment.clone())?;
                    if truth_value.is_truthy() {
                        self.interpret(vec![then_stmt])?
                    } else if let Some(els) = else_stmt {
                        self.interpret(vec![els])?
                    };
                }
                Stmt::Print { expression } => {
                    let result = expression.evaluate(self.environment.clone())?;
                    println!("ECHO: {result}");
                }
                Stmt::Return { keyword: _, value } => {
                    let value = if let Some(expr) = value {
                        expr.evaluate(self.environment.clone())?
                    } else {
                        LiteralValue::Nil
                    };
                    self.specials.insert(String::from("return"), value);
                }
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(self.environment.clone())?;
                    self.environment
                        .borrow_mut()
                        .define(name.lexeme.clone(), value);
                }
                Stmt::While { condition, body } => {
                    let mut flag = condition.evaluate(self.environment.clone())?;
                    while flag.is_truthy() {
                        let statements: Vec<&Stmt> = vec![body.as_ref()];
                        self.interpret(statements)?;
                        flag = condition.evaluate(self.environment.clone())?;
                    }
                }
            };
        }

        Ok(())
    }
}
