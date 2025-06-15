use std::rc::Rc;

use crate::{
    ast::{
        BlockStatement, ClassStatement, ExpressionStatement, FunctionStatement, IfStatement,
        PrintStatement, ReturnStatement, Statement, VarStatement, WhileStatement,
    },
    error::ErrorDetail,
};

use super::{ClassType, FunctionType, Resolve, Scopes};

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

pub fn resolve_function(
    fn_statement: &mut FunctionStatement,
    fn_type: FunctionType,
    scopes: &mut Scopes,
) {
    scopes.begin_function(fn_type);
    scopes.begin_scope();
    for param in &fn_statement.parameters {
        scopes.declare(&param.name, param.line);
        scopes.define(&param.name);
    }
    let mut_statements = Rc::get_mut(&mut fn_statement.statements).unwrap();
    for statement in mut_statements {
        statement.resolve(scopes);
    }
    scopes.end_scope();
    scopes.end_function();
}

impl Resolve for FunctionStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        scopes.declare(&self.name, self.line);
        scopes.define(&self.name);

        resolve_function(self, FunctionType::Function, scopes);
    }
}

impl Resolve for ReturnStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        if let Some(expression) = &mut self.maybe_expression {
            if scopes
                .function_types
                .last()
                .is_some_and(|f| *f == FunctionType::Initializer)
            {
                scopes.errors.push(ErrorDetail::new(
                    self.line,
                    "Can't return a value from an initializer.",
                ));
            }
            expression.resolve(scopes);
        }
        if scopes.function_types.len() == 0 {
            scopes.errors.push(ErrorDetail::new(
                self.line,
                "Can't return from top-level code.",
            ));
        };
    }
}

impl Resolve for ClassStatement {
    fn resolve(&mut self, scopes: &mut Scopes) {
        scopes.begin_class(if self.maybe_superclass.is_some() {
            ClassType::Subclass
        } else {
            ClassType::Class
        });

        scopes.declare(&self.name, self.line);
        scopes.define(&self.name);

        if let Some(superclass) = &mut self.maybe_superclass {
            if superclass.name == self.name {
                scopes.errors.push(ErrorDetail::new(
                    superclass.line,
                    "A class can't inherit from itself.",
                ));
            }
            superclass.resolve(scopes);

            scopes.begin_scope();
            scopes.define("super");
        }

        scopes.begin_scope();
        scopes.define("this");
        for method in Rc::get_mut(&mut self.methods).unwrap().values_mut() {
            let declaration = if method.name == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            resolve_function(method, declaration, scopes);
        }
        // end this scope
        scopes.end_scope();

        if self.maybe_superclass.is_some() {
            scopes.end_scope();
        }

        scopes.end_class();
    }
}
