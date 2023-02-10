use std::rc::Rc;

use crate::{environment::Environment, statement::Stmt};

pub struct Interpreter {
    environment: Rc<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(Environment::new()),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Block { statements } => {
                    let mut new_environment = Environment::new();
                    new_environment.enclosing = Some(self.environment.clone());
                    let old_environment = self.environment.clone();
                    self.environment = Rc::new(new_environment);
                    let block_result = self.interpret(statements);
                    self.environment = old_environment;
                    block_result?
                }
                Stmt::Expression { expression } => {
                    expression.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get a mutable ref to env"),
                    )?;
                }
                Stmt::If {
                    condition,
                    then_stmt,
                    else_stmt,
                } => {
                    let truth_value = condition.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get a mutable ref to env"),
                    )?;
                    if truth_value.is_truthy() {
                        self.interpret(vec![*then_stmt])?
                    } else if let Some(els) = else_stmt {
                        self.interpret(vec![*els])?
                    };
                }
                Stmt::Print { expression } => {
                    let result = expression.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get a mutable ref to env"),
                    )?;
                    println!("ECHO: {}", result.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get a mutable ref to env"),
                    )?;
                    Rc::get_mut(&mut self.environment)
                        .expect("Could not get a mutable ref to env")
                        .define(name.lexeme, value);
                }
            };
        }

        Ok(())
    }
}
