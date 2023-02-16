use crate::expression::Expr;
use crate::scanner::Token;

#[derive(Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
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
                statements.iter().map(|s| s.to_string()).collect::<String>()
            ),
            Stmt::Expression { expression } => expression.to_string(),
            Stmt::Function { name, params, body } => {
                let param_names = params
                    .iter()
                    .map(|p| p.lexeme.clone())
                    .collect::<Vec<String>>();
                let fun_name = &name.lexeme;
                format!("(fun {fun_name} {param_names:?} {body:?})")
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => match else_stmt {
                Some(else_stmt) => {
                    format!("(if {condition} then {then_stmt} else {else_stmt})")
                }
                None => format!("(if {condition} then {then_stmt})"),
            },
            Stmt::Print { expression } => format!("(print {expression})"),
            Stmt::Return { keyword: _, value } => match value {
                Some(expr) => format!("(-> {expr})"),
                None => String::from("(-> nil)"),
            },
            Stmt::Var {
                name,
                initializer: _,
            } => format!("(var {name})"),
            Stmt::While { condition, body } => {
                format!("(while {condition} do {body})")
            }
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
