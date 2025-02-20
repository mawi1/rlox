use std::any::Any;

use crate::{interpreter::Eval, loxtype::LoxType, resolver::Resolve};

pub trait Expression: std::fmt::Debug + Eval + Resolve {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct NilExpression();

#[derive(Debug)]
pub struct LiteralExpression(pub LoxType);

#[derive(Debug)]
pub struct NegExpression {
    pub expression: Box<dyn Expression>,
    pub line: u32,
}

#[derive(Debug)]
pub struct NotExpression(pub Box<dyn Expression>);

#[derive(Debug)]
pub struct GroupingExpression(pub Box<dyn Expression>);

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Substract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: BinaryOperator,
    pub line: u32,
}

#[derive(Debug)]
pub struct VariableExpression {
    pub name: String,
    pub maybe_distance: Option<u32>,
    pub line: u32,
}

#[derive(Debug)]
pub struct AssignExpression {
    pub name: String,
    pub value: Box<dyn Expression>,
    pub maybe_distance: Option<u32>,
    pub line: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug)]
pub struct LogicalExpression {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: LogicalOperator,
}

#[derive(Debug)]
pub struct CallExpression {
    pub callee: Box<dyn Expression>,
    pub arguments: Vec<Box<dyn Expression>>,
    pub line: u32,
}

macro_rules! impl_expression {
    ( $($type:ty),* $(,)? ) => {
        $(
            impl Expression for $type {
                fn as_any(&self) -> &dyn Any {
                    self
                }
            }
        )*
    };
}

impl_expression!(
    NilExpression,
    LiteralExpression,
    NegExpression,
    NotExpression,
    GroupingExpression,
    BinaryExpression,
    VariableExpression,
    AssignExpression,
    LogicalExpression,
    CallExpression
);
