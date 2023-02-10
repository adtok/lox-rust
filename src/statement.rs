use crate::expression::Expr;
use crate::scanner::Token;

pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Expr },
}

impl ToString for Stmt {
    fn to_string(&self) -> String {
        match self {
            Stmt::Expression { expression } => expression.to_string(),
            Stmt::Print { expression } => format!("(print {})", expression.to_string()),
            Stmt::Var {
                name,
                initializer: _,
            } => format!("(var {})", name.to_string()),
        }
    }
}
