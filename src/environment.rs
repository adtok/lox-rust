use std::collections::HashMap;

use crate::{expression::LiteralValue, scanner::Token};

pub struct Environment {
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&LiteralValue> {
        // Handle errors?
        self.values.get(name)
    }

    pub fn assign(&mut self, name: &str, value: LiteralValue) -> bool {
        let old_value = self.get(name);

        match old_value {
            Some(_) => {
                self.values.insert(String::from(name), value);
                true
            }
            _ => false,
        }
    }
}
