// Copyright 2026 Peter Leukanič
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Parser – converts tokens to AST

mod ast;
mod grammar;

pub use ast::*;
pub use grammar::*;

use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Option<std::ops::Range<usize>>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser {
            tokens: lexer.tokens().to_vec(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    // ---------- Token helpers (stays the same) ----------
    fn peek(&self) -> TokenKind {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].kind.clone()
        } else {
            TokenKind::EOF
        }
    }

    fn peek_token(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            static DUMMY_TOKEN: Token = Token {
                kind: TokenKind::EOF,
                span: 0..0,
            };
            return &DUMMY_TOKEN;
        }
        &self.tokens[self.pos]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.peek(), TokenKind::EOF)
    }

    fn advance(&mut self) -> TokenKind {
        let token = self.peek();
        self.pos += 1;
        token
    }

    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.is_at_end() {
            return Err(ParseError {
                message: format!("Unexpected end of input, expected {:?}", expected),
                span: None,
            });
        }

        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.peek()),
                span: Some(self.peek_token().span.clone()),
            })
        }
    }

    fn matches(&mut self, expected: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(code: &str) -> Program {
        let lexer = Lexer::new(code);
        let mut parser = Parser::new(lexer);
        parser.parse().expect("parse failed")
    }

    #[test]
    fn parses_let_statement() {
        let program = parse("let x = 5;");
        assert_eq!(program.statements.len(), 1);
        assert!(
            matches!(&program.statements[0], Stmt::Let { name, .. } if name == "x")
        );
    }

    #[test]
    fn parses_arithmetic_expression() {
        let program = parse("let y = 1 + 2 * 3;");
        assert_eq!(program.statements.len(), 1);
        // Optionally match against Expr::Binary if you expose AST internals
        assert!(matches!(program.statements[0], Stmt::Let { .. }));
    }
}
