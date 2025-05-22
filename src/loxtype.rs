use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    ast::{ClassStatement, FunctionStatement, Statement},
    error::{Error, ErrorDetail},
    interpreter::{run_block, Context, StatementResult},
    Result,
};

pub trait LoxCallable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(&self, arguments: Vec<LoxType>) -> Result<LoxType>;
}

#[derive(Debug)]
pub struct LoxFunction {
    name: String,
    parameters: Vec<String>,
    statements: Rc<Vec<Box<dyn Statement>>>,
    is_initializer: bool,
    ctx: Context,
}

impl LoxFunction {
    pub fn from_statement(stmt: &FunctionStatement, is_initializer: bool, ctx: Context) -> Self {
        Self {
            name: stmt.name.clone(),
            parameters: stmt.parameters.iter().map(|p| p.name.clone()).collect(),
            statements: stmt.statements.clone(),
            is_initializer,
            ctx,
        }
    }
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
        let block_res = run_block(
            self.ctx.clone(),
            &self.statements,
            Some((&self.parameters, arguments)),
        )?;
        if self.is_initializer {
            Ok(self.ctx.get_at(Some(0), "this").unwrap())
        } else {
            match block_res {
                StatementResult::Void => Ok(LoxType::Nil),
                StatementResult::Return(r) => Ok(r),
            }
        }
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, LoxType>,
}

impl LoxInstance {
    fn new(class: Rc<LoxClass>) -> LoxType {
        LoxType::Instance(Rc::new(RefCell::new(Self {
            class: class.clone(),
            fields: HashMap::new(),
        })))
    }

    pub fn get(instance: Rc<RefCell<LoxInstance>>, name: &str, line: u32) -> Result<LoxType> {
        if let Some(field) = instance.borrow().fields.get(name) {
            return Ok(field.clone());
        }

        if let Some(method) = instance.borrow().class.methods.get(name) {
            let child_ctx = instance.borrow().class.ctx.new_child_ctx();
            child_ctx.define("this", LoxType::Instance(instance.clone()));

            let function = LoxFunction::from_statement(method, method.name == "init", child_ctx);
            return Ok(LoxType::Callable(Rc::new(function)));
        }

        Err(Error::RuntimeError(ErrorDetail::new(
            line,
            format!("Undefined property \"{}\".", name),
        )))
    }

    pub fn set(instance: Rc<RefCell<LoxInstance>>, name: &str, value: LoxType) -> LoxType {
        instance
            .borrow_mut()
            .fields
            .insert(name.to_owned(), value.clone());
        value
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}

#[derive(Debug)]
pub struct LoxClass {
    name: String,
    methods: Rc<HashMap<String, FunctionStatement>>,
    ctx: Context,
}

impl LoxClass {
    pub fn from_statement(stmt: &ClassStatement, ctx: Context) -> Self {
        Self {
            name: stmt.name.clone(),
            methods: stmt.methods.clone(),
            ctx,
        }
    }

    pub fn instantiate(self: Rc<Self>, init_arguments: Vec<LoxType>, line: u32) -> Result<LoxType> {
        let instance = LoxInstance::new(self.clone());

        let arity = self.methods.get("init").map_or(0, |im| im.parameters.len());
        if arity != init_arguments.len() {
            return Err(Error::RuntimeError(ErrorDetail::new(
                line,
                format!(
                    "Expected {} arguments but got {}.",
                    arity,
                    init_arguments.len()
                ),
            )));
        }

        if let Some(init_method) = self.methods.get("init") {
            let child_ctx = self.ctx.new_child_ctx();
            child_ctx.define("this", instance.clone());
            let init_method = LoxFunction::from_statement(init_method, true, child_ctx);
            let _ = init_method.call(init_arguments)?;
        }
        Ok(instance)
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub enum LoxType {
    Number(f64),
    Boolean(bool),
    String(String),
    Callable(Rc<dyn LoxCallable>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
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
            LoxType::Class(_) => true,
            LoxType::Instance(_) => true,
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
            (LoxType::Class(l), LoxType::Class(r)) => Rc::ptr_eq(l, r),
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
            LoxType::Class(c) => write!(f, "{c}"),
            LoxType::Instance(i) => write!(f, "{}", i.borrow()),
        }
    }
}
