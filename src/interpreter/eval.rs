use std::rc::Rc;

use crate::{
    ast::*,
    error::{Error, ErrorDetail},
    loxtype::{LoxInstance, LoxType},
    Result,
};

use super::{Context, Eval};

impl Eval for NilExpression {
    fn eval(&self, _: Context) -> Result<LoxType> {
        Ok(LoxType::Nil)
    }
}

impl Eval for LiteralExpression {
    fn eval(&self, _: Context) -> Result<LoxType> {
        Ok(self.0.clone())
    }
}

impl Eval for NegExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        if let LoxType::Number(n) = self.expression.eval(ctx)? {
            Ok(LoxType::Number(-n))
        } else {
            Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                "Operand must be a number.",
            )))
        }
    }
}

impl Eval for NotExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        Ok(LoxType::Boolean(!&self.0.eval(ctx)?.is_truthy()))
    }
}

impl Eval for GroupingExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        self.0.eval(ctx)
    }
}

impl Eval for BinaryExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let left = self.left.eval(ctx.clone())?;
        let right = self.right.eval(ctx)?;

        let incompatible_operands = Err(Error::RuntimeError(ErrorDetail::new(
            self.line,
            "Incompatible operands.",
        )));
        let r = match self.operator {
            BinaryOperator::Add => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Number(l + r),
                (LoxType::String(l), LoxType::String(r)) => LoxType::String(format!("{}{}", l, r)),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::Substract => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Number(l - r),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::Multiply => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Number(l * r),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::Divide => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Number(l / r),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::Equal => LoxType::Boolean(left == right),
            BinaryOperator::NotEqual => LoxType::Boolean(left != right),
            BinaryOperator::Less => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Boolean(l < r),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::LessOrEqual => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Boolean(l <= r),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::Greater => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Boolean(l > r),
                _ => {
                    return incompatible_operands;
                }
            },
            BinaryOperator::GreaterOrEqual => match (left, right) {
                (LoxType::Number(l), LoxType::Number(r)) => LoxType::Boolean(l >= r),
                _ => {
                    return incompatible_operands;
                }
            },
        };
        Ok(r)
    }
}

impl Eval for VariableExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        match ctx.get_at(self.maybe_distance, &self.name) {
            Ok(value) => Ok(value.clone()),
            Err(_) => Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                format!("Undefined variable '{}'.", self.name),
            ))),
        }
    }
}

impl Eval for AssignExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let value = self.value.eval(ctx.clone())?;
        match ctx.assign_at(self.maybe_distance, &self.name, value.clone()) {
            Ok(()) => Ok(value),
            Err(_) => Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                format!("Undefined variable '{}'.", self.name),
            ))),
        }
    }
}

impl Eval for LogicalExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let left = self.left.eval(ctx.clone())?;
        match self.operator {
            LogicalOperator::And => {
                if !left.is_truthy() {
                    Ok(left)
                } else {
                    self.right.eval(ctx)
                }
            }
            LogicalOperator::Or => {
                if left.is_truthy() {
                    Ok(left)
                } else {
                    self.right.eval(ctx)
                }
            }
        }
    }
}

impl Eval for CallExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let callee = self.callee.eval(ctx.clone())?;
        let arguments = self
            .arguments
            .iter()
            .map(|a| a.eval(ctx.clone()))
            .collect::<Result<Vec<LoxType>>>()?;
        if let LoxType::Callable(callable) = callee {
            if callable.arity() != arguments.len() {
                return Err(Error::RuntimeError(ErrorDetail::new(
                    self.line,
                    format!(
                        "Expected {} arguments but got {}.",
                        callable.arity(),
                        arguments.len()
                    ),
                )));
            }
            callable.call(arguments)
        } else if let LoxType::Class(class) = callee {
            class.instantiate(arguments, self.line)
        } else {
            Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                "Can only call functions and classes.",
            )))
        }
    }
}

impl Eval for GetExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let object = self.object.eval(ctx)?;
        if let LoxType::Instance(instance) = object {
            LoxInstance::get(instance, &self.name, self.line)
        } else {
            Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                "Only instances have properties.",
            )))
        }
    }
}

impl Eval for SetExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let object = self.object.eval(ctx.clone())?;
        if let LoxType::Instance(instance) = object {
            let value = self.value.eval(ctx)?;
            Ok(LoxInstance::set(instance, &self.name, value))
        } else {
            Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                "Only instances have fields.",
            )))
        }
    }
}

impl Eval for ThisExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        Ok(ctx.get_at(self.maybe_distance, "this").unwrap())
    }
}

impl Eval for SuperExpression {
    fn eval(&self, ctx: Context) -> Result<LoxType> {
        let superclass = ctx.get_at(self.maybe_distance, "super").unwrap();
        let this: LoxType = ctx
            .get_at(Some(self.maybe_distance.unwrap() - 1), "this")
            .unwrap();

        if let LoxType::Class(sc) = superclass {
            sc.get_method(&self.method, this, self.line).map(|m| LoxType::Callable(Rc::new(m)))
        } else {
            panic!("Superclass is not a class.");
        }
    }
}
