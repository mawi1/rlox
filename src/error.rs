use serde::Serialize;
use std::{borrow::Cow, fmt::Display};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum Error {
    ScannerErrors(Vec<ErrorDetail>),
    SyntaxErrors(Vec<ErrorDetail>),
    ResolverErrors(Vec<ErrorDetail>),
    RuntimeError(ErrorDetail),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ScannerErrors(errors) => {
                writeln!(f, "Scanner error(s):")?;
                for error in errors {
                    writeln!(f, "{error}")?;
                }
            }
            Error::SyntaxErrors(errors) => {
                writeln!(f, "Syntax error(s):")?;
                for error in errors {
                    writeln!(f, "{error}")?;
                }
            }
            Error::RuntimeError(detail) => {
                writeln!(f, "Runtime error: {detail}")?;
            }
            Error::ResolverErrors(errors) => {
                writeln!(f, "Resolver error(s):")?;
                for error in errors {
                    writeln!(f, "{error}")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    line: u32,
    message: Cow<'static, str>,
}

impl ErrorDetail {
    pub fn new(line: u32, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            line: line,
            message: message.into(),
        }
    }
}

impl Display for ErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ line {} ] : {}", self.line, self.message)
    }
}
