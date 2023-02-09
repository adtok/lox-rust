use crate::{
    expression::{Expr, LiteralValue},
    statement::Stmt,
};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluate()?;
                }
                Stmt::Print { expression } => {
                    let result = expression.evaluate()?;
                    println!("{}", result.to_string());
                }
            }
        }

        Ok(())
    }

    pub fn interpret_expr(&self, expr: Expr) -> Result<LiteralValue, String> {
        expr.evaluate()
    }
}
