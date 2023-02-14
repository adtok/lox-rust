use std::cell::RefCell;
use std::rc::Rc;

use crate::{environment::Environment, expression::LiteralValue, statement::Stmt};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

fn clock_impl(_args: &Vec<LiteralValue>) -> LiteralValue {
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
