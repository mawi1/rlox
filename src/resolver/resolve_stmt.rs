use std::rc::Rc;

use crate::ast::{
    BlockStatement, ExpressionStatement, FunctionStatement, IfStatement, PrintStatement,
    ReturnStatement, Statement, VarStatement, WhileStatement,
};

use super::{Resolve, Scopes};

fn resolve_statements(statements: &mut [Box<dyn Statement>], scopes: &mut Scopes) {
    for statement in statements {
        statement.resolve(scopes);
    }
}

impl Resolve for PrintStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.expression.resolve(scopes);
    }
}

impl Resolve for ExpressionStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.0.resolve(scopes);
    }
}

impl Resolve for VarStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        scopes.declare(&self.name, self.line);
        if let Some(i) = self.initializer.as_mut() {
            i.resolve(scopes);
        }
        scopes.define(&self.name);
    }
}

impl Resolve for BlockStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        scopes.begin_scope();
        resolve_statements(&mut self.statements, scopes);
        scopes.end_scope();
    }
}

impl Resolve for IfStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.condition.resolve(scopes);
        self.then_branch.resolve(scopes);
        if let Some(tb) = &mut self.else_branch {
            tb.resolve(scopes);
        }
    }
}

impl Resolve for WhileStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.condition.resolve(scopes);
        self.body.resolve(scopes);
    }
}

impl Resolve for FunctionStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        scopes.declare(&self.name, self.line);
        scopes.define(&self.name);

        scopes.begin_function();
        scopes.begin_scope();
        for param in &self.parameters {
            scopes.declare(&param.name, param.line);
            scopes.define(&param.name);
        }
        let mut_statements = Rc::get_mut(&mut self.statements).unwrap();
        for statement in mut_statements {
            statement.resolve(scopes);
        }
        scopes.end_scope();
        scopes.end_function();
    }
}

impl Resolve for ReturnStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        if let Some(expression) = &mut self.maybe_expression {
            expression.resolve(scopes);
        }
        scopes.check_return_statement(self.line);
    }
}
