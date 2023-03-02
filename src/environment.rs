use std::collections::HashMap;

use crate::expression::LiteralValue;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, LiteralValue>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
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

    pub fn assign(&mut self, name: &str, value: LiteralValue) -> bool {
        let old_value = self.values.get(name);

        match (old_value, &mut self.enclosing) {
            (Some(_), _) => {
                self.values.insert(String::from(name), value);
                true
            }
            (None, Some(env)) => env.assign(name, value),
            (None, None) => false,
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
