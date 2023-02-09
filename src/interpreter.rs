use crate::expression::{Expr, LiteralValue};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: Expr) -> Result<LiteralValue, String> {
        expr.evaluate()
    }
}
