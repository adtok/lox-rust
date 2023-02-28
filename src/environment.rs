use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expression::LiteralValue;

#[derive(Clone)]
pub struct Environment {
    pub values: HashMap<String, LiteralValue>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}
// RcRefCell on values, OptionBox on enclosing??

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

    pub fn get_at(&self, distance: usize, name: &str) -> Option<LiteralValue> {
        if distance == 0 {
            self.values.get(name).cloned()
        } else {
            if let Some(enc_env) = &self.enclosing {
                assert!(distance > 0);
                enc_env.borrow().get_at(distance, name)
            } else {
                panic!("Tried to access ancestor too deep.")
            }
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &str, value: LiteralValue) -> bool {
        if distance == 0 {
            self.assign(name, value)
        } else {
            if let Some(enc_env) = &self.enclosing {
                assert!(distance > 0);
                enc_env.borrow_mut().assign_at(distance - 1, name, value)
            } else {
                panic!("Tried to access ancestor too deep")
            }
        }
    }

    // Should this return a result?
    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        let old_value = self.values.get(name);

        match (old_value, &self.enclosing) {
            (Some(val), _) => Some(val.clone()),
            (_, Some(env)) => env.borrow().get(name),
            (_, _) => None,
        }
    }

    pub fn assign(&mut self, name: &str, value: LiteralValue) -> bool {
        let old_value = self.values.get(name);

        match (old_value, &self.enclosing) {
            (Some(_), _) => {
                self.values.insert(String::from(name), value);
                true
            }
            (None, Some(env)) => env.borrow_mut().assign(name, value),
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
