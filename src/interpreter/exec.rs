use std::rc::Rc;

use crate::{
    ast::{
        BlockStatement, ClassStatement, ExpressionStatement, FunctionStatement, IfStatement,
        PrintStatement, ReturnStatement, VarStatement, WhileStatement,
    },
    error::{Error, ErrorDetail},
    interpreter::Eval,
    loxtype::{LoxClass, LoxFunction, LoxType},
    Result,
};

use super::{run_block, Context, Exec, StatementResult};

impl Exec for PrintStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        let mut out = self.expression.eval(ctx.clone())?.to_string();
        out.push('\n');
        match ctx.write_stdout(&out) {
            Ok(_) => Ok(StatementResult::Void),
            Err(_) => Err(Error::RuntimeError(ErrorDetail::new(
                self.line,
                "Could not write to stdout.",
            ))),
        }
    }
}

impl Exec for ExpressionStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        let _ = self.0.eval(ctx)?;
        Ok(StatementResult::Void)
    }
}

impl Exec for VarStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        let value = match &self.initializer {
            Some(exp) => exp.eval(ctx.clone())?,
            None => LoxType::Nil,
        };
        ctx.define(&self.name, value);
        Ok(StatementResult::Void)
    }
}

impl Exec for BlockStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        run_block(ctx, &self.statements, None)
    }
}

impl Exec for IfStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        if self.condition.eval(ctx.clone())?.is_truthy() {
            self.then_branch.exec(ctx)
        } else {
            if let Some(e) = &self.else_branch {
                e.exec(ctx)
            } else {
                Ok(StatementResult::Void)
            }
        }
    }
}

impl Exec for WhileStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        while self.condition.eval(ctx.clone())?.is_truthy() {
            if let StatementResult::Return(r) = self.body.exec(ctx.clone())? {
                return Ok(StatementResult::Return(r));
            }
        }
        Ok(StatementResult::Void)
    }
}

impl Exec for FunctionStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        let function = LoxFunction::from_statement(self, ctx.clone(), None);
        let callable = LoxType::Callable(Rc::new(function));
        ctx.define(&self.name, callable);
        Ok(StatementResult::Void)
    }
}

impl Exec for ReturnStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        let r = match &self.maybe_expression {
            Some(expression) => expression.eval(ctx)?,
            None => LoxType::Nil,
        };
        Ok(StatementResult::Return(r))
    }
}

impl Exec for ClassStatement {
    fn exec(&self, ctx: Context) -> Result<StatementResult> {
        let maybe_superclass = self
            .maybe_superclass
            .as_ref()
            .map(|superclass_expression| {
                if let LoxType::Class(superclass_class) = superclass_expression.eval(ctx.clone())? {
                    Ok(superclass_class)
                } else {
                    Err(Error::RuntimeError(ErrorDetail::new(
                        superclass_expression.line,
                        "Superclass must be a class.",
                    )))
                }
            })
            .transpose()?;

        ctx.define(&self.name, LoxType::Nil);
        let class = LoxClass::new(self, maybe_superclass, ctx.clone());
        ctx.assign_at(Some(0), &self.name, LoxType::Class(Rc::new(class)))
            .unwrap();
        Ok(StatementResult::Void)
    }
}
