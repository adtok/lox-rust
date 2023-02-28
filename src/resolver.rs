use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expression::Expr;
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::statement::Stmt;

#[derive(Copy, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    current_function: FunctionType,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        let scopes: Vec<HashMap<String, bool>> = vec![];
        Self {
            interpreter: Rc::new(RefCell::new(interpreter)),
            current_function: FunctionType::None,
            scopes,
        }
    }

    pub fn resolve_many(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for statement in statements.iter() {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), String> {
        // "visit"
        match statement.clone() {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_many(statements)?;
                self.end_scope();
            }
            Stmt::Expression { expression } => {
                self.resolve_expr(&expression)?;
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                self.resolve_expr(&condition)?;
                self.resolve_stmt(then_stmt.as_ref())?;
                if let Some(els) = else_stmt {
                    self.resolve_stmt(els.as_ref())?;
                }
            }
            Stmt::Function { name, params, body } => {
                self.declare(&name)?;
                self.define(&name);

                self.resolve_function(&params, &body, FunctionType::Function)?;
            }
            Stmt::Print { expression } => {
                self.resolve_expr(&expression)?;
            }
            Stmt::Return { keyword: _, value } => {
                if self.current_function == FunctionType::None {
                    return Err(String::from("Can't return from top-level code."));
                }

                if let Some(expr) = value {
                    self.resolve_expr(&expr)?;
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(&name)?;
                self.resolve_expr(&initializer)?;
                self.define(&name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(&condition)?;
                self.resolve_stmt(body.as_ref())?;
            }
        };

        Ok(())
    }

    fn resolve_expr(&mut self, expression: &Expr) -> Result<(), String> {
        // "visit"
        match expression {
            Expr::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(expression, name)?;
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                // match callee {
                //     Expr::Variable { name } => self.resolve_local(expression, &name)?,
                //     _ => panic!("Function callee should be Expr::Variable."),
                // }
                self.resolve_expr(callee)?;

                for argument in arguments.iter() {
                    self.resolve_expr(argument)?;
                }
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression)?;
            }
            Expr::Lambda {
                paren: _,
                arguments,
                body,
            } => self.resolve_function(arguments, body, FunctionType::Function)?,
            Expr::Literal { value: _ } => {}
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Unary { operator: _, right } => {
                self.resolve_expr(right)?;
            }
            Expr::Variable { name } => {
                if !self.scopes.is_empty() {
                    if let Some(false) = self.scopes[self.scopes.len() - 1].get(&name.lexeme) {
                        return Err(String::from(
                            "Can't read a variable in its own initializer.",
                        ));
                    }
                }

                self.resolve_local(expression, name)?;
            }
        };
        Ok(())
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        function_type: FunctionType,
    ) -> Result<(), String> {
        let enclosing_function = self.current_function;
        self.current_function = function_type;
        self.begin_scope();
        for param in params.iter() {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_many(body.clone())?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("Scope stack underflow.");
    }

    fn declare(&mut self, name: &Token) -> Result<(), String> {
        let size = self.scopes.len();
        if size == 0 {
            return Ok(());
        }

        if self.scopes[size - 1].contains_key(&name.lexeme.clone()) {
            return Err(format!(
                "A variable with the name '{}' is already in scope.",
                name.lexeme
            ));
        }

        self.scopes[size - 1].insert(name.lexeme.clone(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        let size = self.scopes.len();
        if size == 0 {
            return;
        }

        // if self.scopes[size - 1].contains_key(&name.lexeme) {
        //     panic!("Scope already contains name '{}'.", name.lexeme);
        // }

        self.scopes[size - 1].insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) -> Result<(), String> {
        let size = self.scopes.len();
        if size == 0 {
            return Ok(());
        }

        for i in (0..=(size - 1)).rev() {
            let scope = &self.scopes[i];
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .borrow_mut()
                    .resolve(expression, size - 1 - i)?;
            }
        }

        Ok(())
    }
}
