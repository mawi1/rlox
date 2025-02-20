use crate::ast::{
    AssignExpression, BinaryExpression, CallExpression, GroupingExpression, LiteralExpression,
    LogicalExpression, NegExpression, NilExpression, NotExpression, VariableExpression,
};

use super::{Resolve, Scopes};

impl Resolve for NilExpression {
    fn resolve(&mut self, _scopes: &mut Scopes) {
        ()
    }
}

impl Resolve for LiteralExpression {
    fn resolve(&mut self, _scopes: &mut Scopes) {
        ()
    }
}

impl Resolve for NegExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.expression.resolve(scopes);
    }
}

impl Resolve for NotExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.0.resolve(scopes);
    }
}

impl Resolve for GroupingExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.0.resolve(scopes);
    }
}

impl Resolve for BinaryExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.left.resolve(scopes);
        self.right.resolve(scopes);
    }
}

impl Resolve for VariableExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        scopes.check_initialized(&self.name, self.line);
        self.maybe_distance = scopes.resolve_local(&self.name);
    }
}

impl Resolve for AssignExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.value.resolve(scopes);
        self.maybe_distance = scopes.resolve_local(&self.name);
    }
}

impl Resolve for LogicalExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.left.resolve(scopes);
        self.right.resolve(scopes);
    }
}

impl Resolve for CallExpression {
    fn resolve(&mut self, scopes: &mut Scopes) {
        self.callee.resolve(scopes);
        for arg in &mut self.arguments {
            arg.resolve(scopes);
        }
    }
}
