use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    ast::Statement,
    interpreter::{run_block, Context, StatementResult},
    Result,
};

pub trait LoxCallable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(&self, arguments: Vec<LoxType>) -> Result<LoxType>;
}

#[derive(Debug)]
pub struct LoxFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub statements: Rc<Vec<Box<dyn Statement>>>,
    pub ctx: Context,
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn \"{}\">", self.name)
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(&self, arguments: Vec<LoxType>) -> Result<LoxType> {
        match run_block(
            self.ctx.clone(),
            &self.statements,
            Some((&self.parameters, arguments)),
        )? {
            StatementResult::Void => Ok(LoxType::Nil),
            StatementResult::Return(r) => Ok(r),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoxType {
    Number(f64),
    Boolean(bool),
    String(String),
    Callable(Rc<dyn LoxCallable>),
    Nil,
}

impl LoxType {
    pub fn is_truthy(&self) -> bool {
        match self {
            LoxType::Number(_) => true,
            LoxType::Boolean(b) => *b,
            LoxType::String(_) => true,
            LoxType::Nil => false,
            LoxType::Callable(_) => true,
        }
    }
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoxType::Number(l), LoxType::Number(r)) => l == r,
            (LoxType::String(l), LoxType::String(r)) => l == r,
            (LoxType::Boolean(l), LoxType::Boolean(r)) => l == r,
            (LoxType::Nil, LoxType::Nil) => true,
            (LoxType::Callable(l), LoxType::Callable(r)) => Rc::ptr_eq(l, r),
            _ => false,
        }
    }
}

impl Display for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Number(n) => write!(f, "{n}"),
            LoxType::Boolean(b) => write!(f, "{b}"),
            LoxType::String(s) => write!(f, "{s}"),
            LoxType::Nil => write!(f, "nil"),
            LoxType::Callable(c) => {
                write!(f, "{c}")
            }
        }
    }
}
