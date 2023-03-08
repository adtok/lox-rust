use std::collections::HashMap;

use crate::environment::Environment;
use crate::expression::LiteralValue;
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::statement::Stmt;

#[derive(Clone)]
pub enum LoxCallable {
    LoxFunction {
        name: String,
        parameters: Vec<Token>,
        body: Vec<Stmt>,
        closure: Environment,
    },
    NativeFunction {
        name: String,
        arity: usize,
        fun: CallableFunction,
    },
}

pub type CallableFunction = fn(&Interpreter, &[LiteralValue]) -> Result<LiteralValue, String>;

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            Self::LoxFunction { parameters, .. } => parameters.len(),
            Self::NativeFunction { arity, .. } => arity.clone(),
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralValue],
    ) -> Result<LiteralValue, String> {
        match self {
            Self::LoxFunction {
                name: _,
                parameters,
                body,
                closure,
            } => {
                let args_env: HashMap<_, _> = parameters
                    .iter()
                    .zip(arguments.iter())
                    .map(|(param, arg)| (param.lexeme.clone(), arg.clone()))
                    .collect();

                let saved_env = interpreter.environment.clone();
                let saved_return_value = interpreter.return_value.clone();

                let mut env = closure.clone();
                env.values.extend(saved_env.values.clone());
                env.values.extend(args_env.clone());

                let env = env;
                interpreter.environment = env;
                interpreter.interpret(body)?;
                let return_value = interpreter.return_value.clone();

                interpreter.environment = saved_env;
                interpreter.return_value = saved_return_value;
                match return_value {
                    Some(val) => Ok(val),
                    None => Ok(LiteralValue::Nil),
                }
            }
            Self::NativeFunction { fun, .. } => fun(&interpreter, arguments),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::LoxFunction { name, .. } => name.clone(),
            Self::NativeFunction { name, .. } => name.clone(),
        }
    }
}

impl std::fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}/{}>", self.name(), self.arity())
    }
}
