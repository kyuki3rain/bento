use super::object;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Environment<'a> {
    store: HashMap<String, object::Object<'a>>,
    outer: Option<Box<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        let s = HashMap::new();
        return Environment {
            store: s,
            outer: None,
        };
    }

    pub fn new_enclosed_environment(self) -> Environment<'a> {
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
    pub fn set(&mut self, name: String, val: &object::Object<'a>) {
        self.store.insert(name, val.clone());
    }
}
