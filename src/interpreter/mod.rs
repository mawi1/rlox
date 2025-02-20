mod env;
mod eval;
mod exec;

use std::cell::RefCell;
use std::io::{stdout, Stdout};
use std::rc::Rc;

use crate::ast::Statement;
use crate::loxtype::LoxType;
use crate::native_fns::Clock;
use crate::parser::Parser;
use crate::resolver::resolve;
use crate::scanner::scan_tokens;
use crate::Result;

pub use self::env::{Environment, UndefinedVariable};

pub enum StatementResult {
    Void,
    Return(LoxType),
}

#[derive(Debug, Clone)]
pub struct Context {
    globals: Rc<RefCell<Environment>>,
    env: Rc<RefCell<Environment>>,
    stout: Rc<RefCell<Stdout>>,
    #[cfg(test)]
    test_stout: Rc<RefCell<String>>,
}

impl Context {
    pub fn new() -> Self {
        let globals = Environment::new(None);
        let env = globals.clone();
        Self {
            globals,
            env,
            stout: Rc::new(RefCell::new(stdout())),
            #[cfg(test)]
            test_stout: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn define(&self, name: &str, value: LoxType) {
        self.env.borrow_mut().define(name, value);
    }

    pub fn assign_at(
        &self,
        maybe_distance: Option<u32>,
        name: &str,
        value: LoxType,
    ) -> std::result::Result<(), UndefinedVariable> {
        if let Some(distance) = maybe_distance {
            self.env.borrow_mut().assign_at(distance, name, value)
        } else {
            self.globals.borrow_mut().assign_at(0, name, value)
        }
    }

    pub fn get_at(
        &self,
        maybe_distance: Option<u32>,
        name: &str,
    ) -> std::result::Result<LoxType, UndefinedVariable> {
        if let Some(distance) = maybe_distance {
            self.env.borrow().get_at(distance, name)
        } else {
            self.globals.borrow().get_at(0, name)
        }
    }

    #[cfg(not(test))]
    pub fn write_stdout(&self, t: &str) -> std::result::Result<(), std::io::Error> {
        use std::io::Write;

        let mut out = self.stout.borrow_mut();
        out.write_all(t.as_bytes()).and_then(|_| out.flush())
    }

    #[cfg(test)]
    pub fn write_stdout(&self, t: &str) -> std::result::Result<(), std::io::Error> {
        self.test_stout.borrow_mut().push_str(t);
        Ok(())
    }

    pub fn new_child_ctx(&self) -> Self {
        Context {
            globals: self.globals.clone(),
            env: Environment::new(Some(self.env.clone())),
            stout: self.stout.clone(),
            #[cfg(test)]
            test_stout: self.test_stout.clone(),
        }
    }

    #[cfg(test)]
    pub fn into_writer(self) -> String {
        self.test_stout.borrow().clone()
    }
}

pub trait Eval {
    fn eval(&self, ctx: Context) -> Result<LoxType>;
}

pub trait Exec {
    fn exec(&self, ctx: Context) -> Result<StatementResult>;
}

pub(crate) fn run_block(
    ctx: Context,
    statements: &[Box<dyn Statement>],
    maybe_params_args: Option<(&[String], Vec<LoxType>)>,
) -> crate::Result<StatementResult> {
    let block_ctx = ctx.new_child_ctx();
    if let Some((params, args)) = maybe_params_args {
        assert!(params.len() == args.len(), "");
        for (param, arg) in params.into_iter().zip(args) {
            block_ctx.define(param, arg);
        }
    }
    for statement in statements.iter() {
        if let StatementResult::Return(r) = statement.exec(block_ctx.clone())? {
            return Ok(StatementResult::Return(r));
        }
    }
    Ok(StatementResult::Void)
}
pub struct Interpreter {
    ctx: Context,
}

impl Interpreter {
    pub fn new() -> Self {
        let ctx = Context::new();
        ctx.define("clock", LoxType::Callable(Rc::new(Clock())));
        Self { ctx }
    }

    pub fn run(&self, source: &str) -> Result<()> {
        let tokens = scan_tokens(source)?;
        let mut statements = Parser::new(&tokens).parse()?;
        resolve(&mut statements)?;

        for statement in statements {
            statement.exec(self.ctx.clone())?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn get_output(self) -> String {
        self.ctx.into_writer()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::{assert_snapshot, glob};

    use super::*;

    #[test]
    fn test_interpreter() {
        glob!("../../test_programs/interpreter/", "**/*.lox", |path| {
            let input = fs::read_to_string(path).unwrap();
            let interpreter = Interpreter::new();
            let output = match interpreter.run(&input) {
                Ok(_) => interpreter.get_output(),
                Err(e) => e.to_string(),
            };
            assert_snapshot!(output);
        });
    }
}
