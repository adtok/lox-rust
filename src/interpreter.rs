use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expression::LiteralValue;
use crate::scanner::Token;
use crate::statement::Stmt;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

fn clock_impl(_env: Rc<RefCell<Environment>>, _args: &Vec<LiteralValue>) -> LiteralValue {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Could not get system time.")
        .as_millis();

    LiteralValue::Number(now as f64 / 1000.0)
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();

        globals.define(
            String::from("clock"),
            LiteralValue::Callable {
                name: String::from("clock"),
                arity: 0,
                fun: Rc::new(clock_impl),
            },
        );

        Self {
            environment: Rc::new(RefCell::new(globals)),
        }
    }

    fn for_closure(parent: Rc<RefCell<Environment>>) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().enclosing = Some(parent);

        Self { environment }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Block { statements } => {
                    let mut new_environment = Environment::new();
                    new_environment.enclosing = Some(self.environment.clone());
                    let old_environment = self.environment.clone();
                    self.environment = Rc::new(RefCell::new(new_environment));
                    let block_result =
                        self.interpret((*statements).iter().map(|b| b.as_ref()).collect());
                    self.environment = old_environment;
                    block_result?
                }
                Stmt::Expression { expression } => {
                    expression.evaluate(self.environment.clone())?;
                }
                Stmt::Function { name, params, body } => {
                    let arity = params.len();

                    let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                    let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                    let name_clone = name.lexeme.clone();

                    let fun_impl = move |parent_env, args: &Vec<LiteralValue>| {
                        let mut closure_int = Interpreter::for_closure(parent_env);
                        for (i, arg) in args.iter().enumerate() {
                            closure_int
                                .environment
                                .borrow_mut()
                                .define(params[i].lexeme.clone(), (*arg).clone());
                        }

                        for i in 0..body.len() {
                            closure_int
                                .interpret(vec![body[i].as_ref()])
                                .expect(&format!("Evaluating failed inside {}.", name_clone,));
                        }

                        match body[body.len() - 1].as_ref() {
                            Stmt::Expression { expression } | Stmt::Print { expression } => {
                                expression
                                    .evaluate(closure_int.environment.clone())
                                    .unwrap()
                            }
                            // Stmt::Print { expression } => expression
                            //     .evaluate(closure_int.environment.clone())
                            //     .unwrap(),
                            _ => LiteralValue::Nil,
                        }
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
                    println!("ECHO: {}", result.to_string());
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
