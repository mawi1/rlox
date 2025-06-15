use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::{ast::VariableExpression, interpreter::Exec, resolver::Resolve};

use super::Expression;

pub trait Statement: Debug + Exec + Resolve {}

#[derive(Debug)]
pub struct PrintStatement {
    pub expression: Box<dyn Expression>,
    pub line: u32,
}
impl Statement for PrintStatement {}

#[derive(Debug)]
pub struct ExpressionStatement(pub Box<dyn Expression>);
impl Statement for ExpressionStatement {}

#[derive(Debug)]
pub struct VarStatement {
    pub name: String,
    pub initializer: Option<Box<dyn Expression>>,
    pub line: u32,
}
impl Statement for VarStatement {}

#[derive(Debug)]
pub struct BlockStatement {
    pub statements: Vec<Box<dyn Statement>>,
}
impl Statement for BlockStatement {}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: Box<dyn Expression>,
    pub then_branch: Box<dyn Statement>,
    pub else_branch: Option<Box<dyn Statement>>,
}
impl Statement for IfStatement {}

#[derive(Debug)]
pub struct WhileStatement {
    pub condition: Box<dyn Expression>,
    pub body: Box<dyn Statement>,
}
impl Statement for WhileStatement {}

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub line: u32,
}

#[derive(Debug)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub statements: Rc<Vec<Box<dyn Statement>>>,
    pub line: u32,
}
impl Statement for FunctionStatement {}

#[derive(Debug)]
pub struct ReturnStatement {
    pub maybe_expression: Option<Box<dyn Expression>>,
    pub line: u32,
}
impl Statement for ReturnStatement {}

#[derive(Debug)]
pub struct ClassStatement {
    pub name: String,
    pub methods: Rc<HashMap<String, FunctionStatement>>,
    pub maybe_superclass: Option<VariableExpression>,
    pub line: u32,
}
impl Statement for ClassStatement {}
