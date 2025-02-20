use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{LoxCallable, LoxType};

#[derive(Debug)]
pub struct Clock();

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn clock>")
    }
}

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _arguments: Vec<LoxType>) -> crate::Result<LoxType> {
        let now = SystemTime::now();
        let elapsed = now.duration_since(UNIX_EPOCH).unwrap();
        Ok(LoxType::Number(elapsed.as_secs() as f64))
    }
}
