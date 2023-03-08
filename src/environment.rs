use std::collections::HashMap;

use crate::expression::LiteralValue;
use crate::scanner::Token;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, LiteralValue>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn define(&mut self, name: Token, value: LiteralValue) {
        self.values.insert(name.lexeme, value);
    }

    // Should this return a result?
    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        let old_value = self.values.get(name);

        match (old_value, &self.enclosing) {
            (Some(val), _) => Some(val.clone()),
            (_, Some(env)) => env.get(name),
            (_, _) => None,
        }
    }

    pub fn assign(&mut self, token: Token, value: &LiteralValue) -> Result<(), String> {
        if self.values.contains_key(&token.lexeme) {
            self.define(token, value.clone());
            return Ok(());
        }

        match &mut self.enclosing {
            Some(enclosing) => enclosing.assign(token, value),
            None => Err(format!(
                "Attempting to assign to variable '{}' that does not exist",
                token.lexeme.clone()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_environment() {
        let _environment = Environment::new();
    }
}
