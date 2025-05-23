mod resolve_expr;
mod resolve_stmt;

use std::collections::HashMap;

use crate::ast::Statement;
use crate::error::{Error, ErrorDetail};
use crate::Result;

#[derive(Debug, PartialEq, Eq)]
enum FunctionType {
    Function,
    Initializer,
    Method,
}

#[derive(Debug, PartialEq, Eq)]
enum ClassType {
    Class,
    Subclass,
}

#[derive(Debug, PartialEq, Eq)]
enum VariableState {
    Declared,
    Defined,
}

pub(crate) struct Scopes {
    scopes: Vec<HashMap<String, VariableState>>,
    function_types: Vec<FunctionType>,
    class_types: Vec<ClassType>,
    errors: Vec<ErrorDetail>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: vec![],
            function_types: vec![],
            class_types: vec![],
            errors: vec![],
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub(self) fn begin_function(&mut self, fn_type: FunctionType) {
        self.function_types.push(fn_type);
    }

    pub fn end_function(&mut self) {
        self.function_types.pop();
    }

    pub(self) fn begin_class(&mut self, class_type: ClassType) {
        self.class_types.push(class_type);
    }

    pub fn end_class(&mut self) {
        self.class_types.pop();
    }

    pub fn declare(&mut self, name: &str, line: u32) {
        if let Some(hm) = self.scopes.last_mut() {
            if hm.contains_key(name) {
                self.errors.push(ErrorDetail::new(
                    line,
                    "Already a variable with this name in this scope.",
                ));
            } else {
                hm.insert(name.to_owned(), VariableState::Declared);
            }
        }
    }

    pub fn define(&mut self, name: &str) {
        if let Some(hm) = self.scopes.last_mut() {
            hm.insert(name.to_owned(), VariableState::Defined);
        }
    }

    pub fn check_initialized(&mut self, name: &str, line: u32) {
        if self
            .scopes
            .last()
            .is_some_and(|hm| hm.get(name).is_some_and(|v| *v == VariableState::Declared))
        {
            self.errors.push(ErrorDetail::new(
                line,
                "Can't read local variable in its own initializer.",
            ));
        }
    }

    pub fn resolve_local(&self, name: &str) -> Option<u32> {
        self.scopes
            .iter()
            .rev()
            .position(|hm| hm.contains_key(name))
            .map(|v| v as u32)
    }

    pub fn into_errors(self) -> Vec<ErrorDetail> {
        self.errors
    }
}

pub trait Resolve {
    fn resolve(&mut self, scopes: &mut Scopes);
}

pub fn resolve(statements: &mut [Box<dyn Statement>]) -> Result<()> {
    let mut scopes = Scopes::new();
    for statement in statements {
        statement.resolve(&mut scopes)
    }

    let errors = scopes.into_errors();
    if errors.len() > 0 {
        Err(Error::ResolverErrors(errors))
    } else {
        Ok(())
    }
}
