use std::rc::Rc;

use crate::{expression::LiteralValue, interpreter::Interpreter};

#[derive(Clone)]
pub enum LoxCallable {
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
