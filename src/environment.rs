use super::object;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Environment {
    store: HashMap<String, object::Object>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        let s = HashMap::new();
        return Environment {
            store: s,
            outer: None,
        };
    }

    pub fn new_enclosed_environment(self) -> Environment {
        return Environment {
            store: HashMap::new(),
            outer: Some(Box::new(self)),
        };
    }

    pub fn get(&self, name: String) -> Option<object::Object> {
        match self.store.get(&name) {
            Some(value) => return Some(value.clone()),
            None => match &self.outer {
                Some(out_env) => return out_env.get(name),
                None => return None,
            },
        }
    }
    pub fn set(&mut self, name: String, val: &object::Object) {
        self.store.insert(name, val.clone());
    }
}
