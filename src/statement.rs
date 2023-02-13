use crate::expression::Expr;
use crate::scanner::Token;

pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Expression {
        expression: Expr,
    },
    If {
        condition: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Stmt::Block { statements } => format!(
                "(block {})",
                statements
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<String>()
            ),
            Stmt::Expression { expression } => expression.to_string(),
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => match else_stmt {
                Some(else_stmt) => format!(
                    "(if {} then {} else {})",
                    condition.to_string(),
                    then_stmt.to_string(),
                    else_stmt.to_string()
                ),
                None => format!(
                    "(if {} then {})",
                    condition.to_string(),
                    then_stmt.to_string()
                ),
            },
            Stmt::Print { expression } => format!("(print {})", expression.to_string()),
            Stmt::Var {
                name,
                initializer: _,
            } => format!("(var {})", name.to_string()),
            Stmt::While { condition, body } => {
                format!("(while {} do {})", condition.to_string(), body.to_string())
            }
        };
        write!(f, "{}", s)
    }
}
