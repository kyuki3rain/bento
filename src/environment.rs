use super::object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub struct Environment {
    store: HashMap<String, Rc<object::Object>>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        let mut s = HashMap::new();
        s.insert("null".to_string(), Rc::new(object::NULL));
        s.insert("true".to_string(), Rc::new(object::TRUE));
        s.insert("false".to_string(), Rc::new(object::FALSE));
        s.insert("exit".to_string(), Rc::new(object::EXIT));
        return Environment {
            store: s,
            outer: None,
        };
    }

    pub fn new_enclosed_environment(outer: Rc<RefCell<Environment>>) -> Environment {
        return Environment {
            store: HashMap::new(),
            outer: Some(outer),
        };
    }

    pub fn get(&self, name: String) -> Option<Rc<object::Object>> {
        match self.store.get(&name) {
            Some(value) => return Some(Rc::clone(value)),
            None => match &self.outer {
                Some(out_env) => return out_env.borrow_mut().get(name),
                None => return None,
            },
        }
    }
    pub fn set(&mut self, name: String, val: Rc<object::Object>) {
        self.store.insert(name, val);
    }
    pub fn contains_key(&mut self, name: &str) -> bool {
        return self.store.contains_key(name);
    }
}
