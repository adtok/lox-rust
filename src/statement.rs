use crate::expression::Expr;

pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
}
