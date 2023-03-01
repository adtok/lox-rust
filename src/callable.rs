use std::rc::Rc;

use crate::environment::Environment;
use crate::expression::LiteralValue;
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::statement::Stmt;

#[derive(Clone)]
pub enum LoxCallable {
    LoxFunction {
        name: String,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Environment,
    },
    NativeFunction {
        name: String,
        arity: usize,
        fun: CallableFunction,
    },
}

pub type CallableFunction = Rc<dyn Fn(&[LiteralValue]) -> LiteralValue>;

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            Self::LoxFunction {
                name: _,
                params,
                body: _,
                closure: _,
            } => params.len(),
            Self::NativeFunction {
                name: _,
                arity,
                fun: _,
            } => arity.clone(),
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LiteralValue>,
    ) -> Result<LiteralValue, String> {
        let result = match self {
            Self::LoxFunction {
                name: _,
                params: _,
                body: _,
                closure: _,
            } => todo!(),
            Self::NativeFunction {
                name: _,
                arity: _,
                fun,
            } => fun(&arguments),
        };
        Ok(result)
    }

    pub fn name(&self) -> String {
        match self {
            Self::LoxFunction {
                name,
                params: _,
                body: _,
                closure: _,
            } => name.clone(),
            Self::NativeFunction {
                name,
                arity: _,
                fun: _,
            } => name.clone(),
        }
    }
}

impl std::fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}/{}>", self.name(), self.arity())
    }
}
