use std::{iter::Peekable, rc::Rc};

use crate::{
    ast::*,
    error::{Error, ErrorDetail},
    loxtype::LoxType,
    token::{
        Literal, Token,
        TokenType::{self, *},
    },
    Result,
};

#[derive(Debug)]
enum FunctionKind {
    Function,
    #[allow(dead_code)]
    Method,
}

pub struct Parser<'a> {
    tokens: Peekable<std::iter::Take<std::slice::Iter<'a, Token>>>,
    errors: Vec<ErrorDetail>,
    last_line: u32,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            // iterate without Eof token at end
            tokens: tokens.iter().take(tokens.len() - 1).peekable(),
            errors: Vec::new(),
            last_line: tokens
                .get(tokens.len().wrapping_sub(2))
                .map(|t| t.line)
                .unwrap_or(1),
        }
    }

    pub fn parse(mut self) -> Result<Vec<Box<dyn Statement>>> {
        let mut statements = vec![];

        while self.tokens.peek().is_some() {
            match self.declaration() {
                Ok(s) => statements.push(s),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize()
                }
            }
        }

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(Error::SyntaxErrors(self.errors))
        }
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.tokens.peek() {
            let ty = token.ty;
            if ty == Semicolon {
                self.tokens.next();
                return;
            }
            if [Class, Fun, Var, For, If, While, Print, Return]
                .iter()
                .any(|&tt| tt == ty)
            {
                return;
            };
            self.tokens.next();
        }
    }

    fn match_token_type(&mut self, tt: TokenType) -> Option<&'a Token> {
        self.tokens.next_if(|t| t.ty == tt)
    }

    fn match_token_types(&mut self, tts: &[TokenType]) -> Option<&'a Token> {
        if let Some(t) = self.tokens.peek() {
            if tts.iter().any(|&tt| tt == t.ty) {
                Some(self.tokens.next().unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_next_token_type(&mut self, tt: TokenType) -> bool {
        self.tokens.next_if(|t| t.ty == tt).is_some()
    }

    fn consume(&mut self, token_ty: TokenType) -> std::result::Result<&'a Token, ErrorDetail> {
        if let Some(n) = self.tokens.peek() {
            if n.ty == token_ty {
                Ok(self.tokens.next().unwrap())
            } else {
                Err(ErrorDetail::new(n.line, format!("Expect '{token_ty}'.")))
            }
        } else {
            Err(ErrorDetail::new(
                self.last_line,
                format!("Expect '{token_ty}'."),
            ))
        }
    }

    fn declaration(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        match self.tokens.peek().unwrap().ty {
            Var => self.var_declaration(),
            Fun => self.function(FunctionKind::Function),
            _ => self.statement(),
        }
    }

    fn function(
        &mut self,
        kind: FunctionKind,
    ) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        let fun_token = self.tokens.next().unwrap();
        if let Some(name_token) = self.tokens.next_if(|t| t.ty == Identifier) {
            let name = name_token.lexeme.clone();

            self.consume(LeftParen)?;
            let mut parameters = vec![];
            if self.tokens.peek().is_some_and(|t| t.ty != RightParen) {
                loop {
                    let identifier = self.consume(Identifier)?;
                    parameters.push(Parameter {
                        name: identifier.lexeme.clone(),
                        line: identifier.line,
                    });
                    if !self.is_next_token_type(Comma) {
                        break;
                    }
                }
            }
            let paren_token = self.consume(RightParen)?;
            if parameters.len() > 255 {
                self.errors.push(ErrorDetail::new(
                    paren_token.line,
                    "Can't have more than 255 parameters.",
                ));
            }

            self.consume(LeftBrace)?;
            let block = self.block_statement()?;

            Ok(Box::new(FunctionStatement {
                name,
                parameters,
                statements: Rc::new(block.statements),
                line: fun_token.line,
            }))
        } else {
            let message = match kind {
                FunctionKind::Function => "Expect function name.",
                FunctionKind::Method => "Expect method name.",
            };
            Err(ErrorDetail::new(fun_token.line, message))
        }
    }

    fn var_declaration(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        let var_token = self.tokens.next().unwrap();
        let name = self.consume(Identifier)?;

        let initializer = if self.is_next_token_type(Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Semicolon)?;

        Ok(Box::new(VarStatement {
            name: name.lexeme.clone(),
            initializer: initializer,
            line: var_token.line,
        }))
    }

    fn statement(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        match self.tokens.peek().unwrap().ty {
            For => {
                self.tokens.next();
                self.for_statement()
            }
            If => {
                self.tokens.next();
                self.if_statement()
            }
            LeftBrace => {
                self.tokens.next();
                self.block_statement()
                    .map(|b| Box::new(b) as Box<dyn Statement>)
            }
            Print => self.print_statement(),
            Return => self.return_statemen(),
            While => {
                self.tokens.next();
                self.while_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn return_statemen(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        let return_token = self.tokens.next().unwrap();
        let maybe_expression = match self.tokens.peek().is_some_and(|t| t.ty != Semicolon) {
            true => Some(self.expression()?),
            false => None,
        };
        self.consume(Semicolon)?;
        Ok(Box::new(ReturnStatement {
            maybe_expression,
            line: return_token.line,
        }))
    }

    fn for_statement(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        self.consume(LeftParen)?;

        let opt_initializer = if self.is_next_token_type(Semicolon) {
            None
        } else if self.tokens.peek().is_some_and(|t| t.ty == Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let opt_for_condition = if self.is_next_token_type(Semicolon) {
            None
        } else {
            let e = self.expression()?;
            self.consume(Semicolon)?;
            Some(e)
        };

        let opt_increment = if self.is_next_token_type(RightParen) {
            None
        } else {
            let i = self.expression()?;
            self.consume(RightParen)?;
            Some(i)
        };

        let for_body = self.statement()?;

        //desugar as while-loop:
        //{
        // initializer;
        // while(condition) {
        //  body;
        //  increment;
        // }
        //}
        let condition =
            opt_for_condition.unwrap_or(Box::new(LiteralExpression(LoxType::Boolean(true))));

        let mut body_statements: Vec<Box<dyn Statement>> = vec![for_body];
        if let Some(increment) = opt_increment {
            body_statements.push(Box::new(ExpressionStatement(increment)));
        }
        let body = Box::new(BlockStatement {
            statements: body_statements,
        });

        let while_statement = Box::new(WhileStatement { condition, body });
        let mut block_statements: Vec<Box<dyn Statement>> = vec![];
        if let Some(initializer) = opt_initializer {
            block_statements.push(initializer);
        }
        block_statements.push(while_statement);

        Ok(Box::new(BlockStatement {
            statements: block_statements,
        }))
    }

    fn while_statement(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        self.consume(LeftParen)?;
        let condition = self.expression()?;
        self.consume(RightParen)?;
        let body = self.statement()?;
        Ok(Box::new(WhileStatement { condition, body }))
    }

    fn if_statement(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        self.consume(LeftParen)?;
        let condition = self.expression()?;
        self.consume(RightParen)?;

        let then_branch = self.statement()?;
        let else_branch = self
            .match_token_type(Else)
            .map(|_| self.statement())
            .transpose()?;
        Ok(Box::new(IfStatement {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn print_statement(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        let print_token = self.tokens.next().unwrap();
        let expression = self.expression()?;
        self.consume(Semicolon)?;
        Ok(Box::new(PrintStatement {
            expression,
            line: print_token.line,
        }))
    }

    fn block_statement(&mut self) -> std::result::Result<BlockStatement, ErrorDetail> {
        let mut statements = Vec::new();

        while let Some(token) = self.tokens.peek() {
            if token.ty == RightBrace {
                break;
            } else {
                statements.push(self.declaration()?);
            }
        }

        self.consume(RightBrace)?;
        Ok(BlockStatement { statements })
    }

    fn expression_statement(&mut self) -> std::result::Result<Box<dyn Statement>, ErrorDetail> {
        let e = self.expression()?;
        self.consume(Semicolon)?;
        Ok(Box::new(ExpressionStatement(e)))
    }

    fn expression(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        self.assignment()
    }

    fn assignment(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let expr = self.or()?;

        if let Some(eq_token) = self.match_token_type(Equal) {
            let value = self.assignment()?;

            let expr_any = expr.as_any();
            match expr_any.downcast_ref::<VariableExpression>() {
                Some(var_expr) => {
                    return Ok(Box::new(AssignExpression {
                        name: var_expr.name.clone(),
                        value: value,
                        maybe_distance: None,
                        line: eq_token.line,
                    }));
                }
                None => {
                    self.errors.push(ErrorDetail::new(
                        eq_token.line,
                        "Invalid assignment target.",
                    ));
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.and()?;

        while self.is_next_token_type(Or) {
            let right = self.and()?;
            expr = Box::new(LogicalExpression {
                left: expr,
                right: right,
                operator: LogicalOperator::Or,
            });
        }
        return Ok(expr);
    }

    fn and(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.equality()?;

        while self.is_next_token_type(And) {
            let right = self.equality()?;
            expr = Box::new(LogicalExpression {
                left: expr,
                right: right,
                operator: LogicalOperator::And,
            });
        }
        return Ok(expr);
    }

    fn equality(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.match_token_types(&[BangEqual, EqualEqual]) {
            let right = self.comparison()?;
            expr = match operator.ty {
                BangEqual => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::NotEqual,
                    line: operator.line,
                }),
                EqualEqual => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Equal,
                    line: operator.line,
                }),
                _ => unreachable!(),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.term()?;

        while let Some(operator) = self.match_token_types(&[Greater, GreaterEqual, Less, LessEqual])
        {
            let right = self.term()?;
            expr = match operator.ty {
                Greater => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Greater,
                    line: operator.line,
                }),
                GreaterEqual => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::GreaterOrEqual,
                    line: operator.line,
                }),
                Less => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Less,
                    line: operator.line,
                }),
                LessEqual => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::LessOrEqual,
                    line: operator.line,
                }),
                _ => unreachable!(),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.match_token_types(&[Minus, Plus]) {
            let right = self.factor()?;
            expr = match operator.ty {
                Minus => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Substract,
                    line: operator.line,
                }),
                Plus => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Add,
                    line: operator.line,
                }),
                _ => unreachable!(),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.match_token_types(&[Star, Slash]) {
            let right = self.unary()?;
            expr = match operator.ty {
                Star => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Multiply,
                    line: operator.line,
                }),
                Slash => Box::new(BinaryExpression {
                    left: expr,
                    right: right,
                    operator: BinaryOperator::Divide,
                    line: operator.line,
                }),
                _ => unreachable!(),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        if let Some(operator) = self.match_token_types(&[Bang, Minus]) {
            let expression = self.unary()?;

            return Ok(match operator.ty {
                Bang => Box::new(NotExpression(expression)),
                Minus => Box::new(NegExpression {
                    expression,
                    line: operator.line,
                }),
                _ => unreachable!(),
            });
        }

        self.call()
    }

    fn finish_call(
        &mut self,
        callee: Box<dyn Expression>,
    ) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut arguments = vec![];

        if self.tokens.peek().is_some_and(|t| t.ty != RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.is_next_token_type(Comma) {
                    break;
                }
            }
        }
        let paren_token = self.consume(RightParen)?;
        if arguments.len() > 255 {
            self.errors.push(ErrorDetail::new(
                paren_token.line,
                "Can't have more than 255 arguments.",
            ));
        }

        Ok(Box::new(CallExpression {
            callee,
            arguments,
            line: paren_token.line,
        }))
    }

    fn call(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        let mut expr = self.primary()?;

        loop {
            if self.is_next_token_type(LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> std::result::Result<Box<dyn Expression>, ErrorDetail> {
        if let Some(token) = self.tokens.next() {
            match token.ty {
                Nil => Ok(Box::new(NilExpression())),
                True => Ok(Box::new(LiteralExpression(LoxType::Boolean(true)))),
                False => Ok(Box::new(LiteralExpression(LoxType::Boolean(false)))),
                Number => {
                    if let Literal::Number(n) = token.literal.as_ref().expect("no literal value") {
                        Ok(Box::new(LiteralExpression(LoxType::Number(*n))))
                    } else {
                        panic!("literal type mismatch");
                    }
                }
                String => {
                    if let Literal::String(s) = token.literal.as_ref().expect("no literal value") {
                        Ok(Box::new(LiteralExpression(LoxType::String(s.clone()))))
                    } else {
                        panic!("literal type mismatch");
                    }
                }
                LeftParen => {
                    let expr = self.expression()?;
                    self.consume(RightParen)?;
                    Ok(Box::new(GroupingExpression(expr)))
                }
                Identifier => Ok(Box::new(VariableExpression {
                    name: token.lexeme.clone(),
                    maybe_distance: None,
                    line: token.line,
                })),
                _ => Err(ErrorDetail::new(token.line, "Expect expression.")),
            }
        } else {
            Err(ErrorDetail::new(self.last_line, "Expect expression."))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::{assert_debug_snapshot, glob};

    use crate::scanner::scan_tokens;

    use super::*;

    #[test]
    fn test_parser() {
        glob!("../test_programs/parsing/", "**/*.lox", |path| {
            let input = fs::read_to_string(path).unwrap();
            let tokens = scan_tokens(&input).unwrap();
            let parser = Parser::new(&tokens);
            assert_debug_snapshot!(parser.parse());
        });
    }
}
