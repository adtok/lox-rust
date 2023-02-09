use crate::expression::Expr;
use crate::scanner::Token;

pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Expr },
}
