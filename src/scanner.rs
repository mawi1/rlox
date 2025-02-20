use itertools::Itertools;
use phf::phf_map;

use crate::error::{Error, ErrorDetail};
use crate::token::{
    Literal, Token,
    TokenType::{self, *},
};
use crate::Result;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => And,
    "class" => Class,
    "else" => Else,
    "false" => False,
    "for" => For,
    "fun" => Fun,
    "if" => If,
    "nil" => Nil,
    "or" => Or,
    "print" => Print,
    "return" => Return,
    "super" => Super,
    "this" => This,
    "true" => True,
    "var" => Var,
    "while" => While,
};

pub fn scan_tokens(source: &str) -> Result<Vec<Token>> {
    let mut tokens = vec![];
    let mut errors = vec![];
    let mut line = 1;

    let mut chars = source.chars().multipeek();
    while let Some(c) = chars.next() {
        let mut add_token = |ty: TokenType| tokens.push(Token::new(ty, c.to_string(), None, line));

        match c {
            // one char tokens
            '(' => add_token(LeftParen),
            ')' => add_token(RightParen),
            '{' => add_token(LeftBrace),
            '}' => add_token(RightBrace),
            ',' => add_token(Comma),
            '.' => add_token(Dot),
            '-' => add_token(Minus),
            '+' => add_token(Plus),
            ';' => add_token(Semicolon),
            '*' => add_token(Star),
            // two char tokens
            '!' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(BangEqual, "!=".to_owned(), None, line));
                } else {
                    tokens.push(Token::new(Bang, c.to_string(), None, line));
                }
            }
            '=' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(EqualEqual, "==".to_owned(), None, line));
                } else {
                    tokens.push(Token::new(Equal, c.to_string(), None, line));
                }
            }
            '<' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(LessEqual, "<=".to_owned(), None, line));
                } else {
                    tokens.push(Token::new(Less, c.to_string(), None, line));
                }
            }
            '>' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::new(GreaterEqual, ">=".to_owned(), None, line));
                } else {
                    tokens.push(Token::new(Greater, c.to_string(), None, line));
                }
            }
            // comment or slash
            '/' => {
                if let Some('/') = chars.peek() {
                    chars.next();
                    while let Some(&next_char) = chars.peek() {
                        if next_char == '\n' {
                            break;
                        } else {
                            chars.next();
                        }
                    }
                } else {
                    tokens.push(Token::new(Slash, c.to_string(), None, line));
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => line += 1,
            '"' => {
                let mut string_string = std::string::String::new();

                while chars.peek().is_some_and(|c| *c != '"') {
                    let next_char = chars.next().unwrap();
                    if next_char == '\n' {
                        line += 1;
                        dbg!(line);
                    }
                    string_string.push(next_char);
                }

                if chars.peek().is_none() {
                    errors.push(ErrorDetail::new(line, "Unterminated string."));
                    break;
                }

                chars.next(); // consume closing "

                tokens.push(Token::new(
                    String,
                    string_string.clone(),
                    Some(Literal::String(string_string)),
                    line,
                ));
            }
            _ => {
                if c.is_ascii_digit() {
                    let mut num_string = c.to_string();

                    while chars.peek().is_some_and(|pc| pc.is_ascii_digit()) {
                        let t = chars.next().unwrap();
                        num_string.push(t);
                    }

                    chars.reset_peek();
                    let maybe_dot = chars.peek().cloned();
                    let maybe_digit = chars.peek().cloned();
                    if maybe_dot.is_some_and(|md| md == '.')
                        && maybe_digit.is_some_and(|md| md.is_ascii_digit())
                    {
                        num_string.push(chars.next().unwrap()); // consume '.'

                        while chars.peek().is_some_and(|pc| pc.is_ascii_digit()) {
                            num_string.push(chars.next().unwrap());
                        }
                    }

                    let parse_res = num_string.parse::<f64>();
                    if let Err(_) = parse_res {
                        errors.push(ErrorDetail::new(
                            line,
                            format!("Could not parse number: {num_string}."),
                        ));
                        continue;
                    }

                    tokens.push(Token::new(
                        Number,
                        num_string,
                        Some(Literal::Number(parse_res.unwrap())),
                        line,
                    ));
                } else if c.is_ascii_alphabetic() || c == '_' {
                    let mut identifier_string = c.to_string();

                    while chars
                        .peek()
                        .is_some_and(|pc| pc.is_ascii_alphanumeric() || *pc == '_')
                    {
                        identifier_string.push(chars.next().unwrap());
                    }

                    if let Some(ty) = KEYWORDS.get(&identifier_string) {
                        tokens.push(Token::new(*ty, identifier_string, None, line));
                    } else {
                        tokens.push(Token::new(Identifier, identifier_string, None, line));
                    }
                } else {
                    errors.push(ErrorDetail::new(
                        line,
                        format!("Unexpected character: {c}."),
                    ));
                }
            }
        }
    }
    tokens.push(Token::new(Eof, "".to_string(), None, line));

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(Error::ScannerErrors(errors))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::{assert_debug_snapshot, glob};

    use super::*;

    #[test]
    fn test_scanner() {
        glob!("../test_programs/scanning/", "*.lox", |path| {
            let input = fs::read_to_string(path).unwrap();
            assert_debug_snapshot!(scan_tokens(&input));
        });
    }
}
