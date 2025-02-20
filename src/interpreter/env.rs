use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::loxtype::LoxType;

#[derive(Debug, PartialEq, Eq)]
pub struct UndefinedVariable();

#[derive(Debug)]
pub struct Environment {
    maybe_enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, LoxType>,
}

impl Environment {
    pub fn new(maybe_enclosing: Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            maybe_enclosing,
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: &str, value: LoxType) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign_at(
        &mut self,
        distance: u32,
        name: &str,
        value: LoxType,
    ) -> Result<(), UndefinedVariable> {
        if distance == 0 {
            if self.values.contains_key(name) {
                self.values.insert(name.to_owned(), value);
                Ok(())
            } else {
                Err(UndefinedVariable())
            }
        } else {
            if let Some(enclosing) = &self.maybe_enclosing {
                enclosing.borrow_mut().assign_at(distance - 1, name, value)
            } else {
                panic!(
                    "line {}: could not assign variable {} at distance {}",
                    33, name, distance
                )
            }
        }
    }

    pub fn get_at(&self, distance: u32, name: &str) -> Result<LoxType, UndefinedVariable> {
        if distance == 0 {
            self.values
                .get(name)
                .map(|v| v.clone())
                .ok_or(UndefinedVariable())
        } else {
            if let Some(enclosing) = &self.maybe_enclosing {
                enclosing.borrow().get_at(distance - 1, name)
            } else {
                panic!(
                    "line {}: could not read variable {} at distance {}",
                    33, name, distance
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_env() -> Rc<RefCell<Environment>> {
        let global = Environment::new(None);
        global.borrow_mut().define("a", LoxType::Number(1.0));
        let e1 = Environment::new(Some(global));
        let e2 = Environment::new(Some(e1));
        e2
    }

    #[test]
    fn test_get() {
        let env = test_env();
        let n = env.borrow().get_at(2, "a").unwrap();
        assert_eq!(n, LoxType::Number(1.0));
    }

    #[test]
    fn test_get_undefined() {
        let env = test_env();
        let e = env.borrow().get_at(2, "c").unwrap_err();
        assert_eq!(e, UndefinedVariable());
    }

    #[test]
    fn test_assign() {
        let env = test_env();
        env.borrow_mut()
            .assign_at(2, "a", LoxType::Boolean(false))
            .unwrap();
        let v = env.borrow().get_at(2, "a").unwrap();
        assert_eq!(v, LoxType::Boolean(false));
    }

    #[test]
    fn test_assign_undefined() {
        let env = test_env();
        let e = env
            .borrow_mut()
            .assign_at(2, "c", LoxType::Boolean(false))
            .unwrap_err();
        assert_eq!(e, UndefinedVariable());
    }

    #[test]
    fn test_define() {
        let env = test_env();
        env.borrow_mut().define("foo", LoxType::Nil);
        let v = env.borrow().get_at(0, "foo").unwrap();
        assert_eq!(v, LoxType::Nil);
    }
}
