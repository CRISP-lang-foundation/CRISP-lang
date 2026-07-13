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

//! Grammar – statement and expression parsing for CRISP

use super::Parser;
use crate::lexer::TokenKind;
use crate::parser::{BinaryOp,
		    Expr,
		    MatchArm,
		    MatchPattern,
		    Param,
		    ParseError,
		    Stmt,
		    UnaryOp,
		    VarType};

impl Parser {
    /// Main statement dispatcher
    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Const => self.parse_const_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
	    TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            TokenKind::Print => {
                self.advance(); // consume 'print'
                if self.peek() == TokenKind::LParen {
                    // Function-call syntax: print("hello", name)
                    self.advance(); // consume '('
                    let mut exprs = Vec::new();
                    if self.peek() != TokenKind::RParen {
                        loop {
                            exprs.push(self.parse_expression()?);
                            if self.peek() == TokenKind::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Print(exprs))
                } else {
                    // Bare syntax: print "hello", name
                    let mut exprs = vec![self.parse_expression()?];
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        exprs.push(self.parse_expression()?);
                    }
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Print(exprs))
                }
            }
            TokenKind::Say => {
                self.advance(); // consume 'say'
                if self.peek() == TokenKind::LParen {
                    // Function-call syntax: say("hello", name)
                    self.advance(); // consume '('
                    let mut exprs = Vec::new();
                    if self.peek() != TokenKind::RParen {
                        loop {
                            exprs.push(self.parse_expression()?);
                            if self.peek() == TokenKind::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Say(exprs))
                } else {
                    // Bare syntax: say "hello", name
                    let mut exprs = vec![self.parse_expression()?];
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        exprs.push(self.parse_expression()?);
                    }
                    self.maybe_consume_semicolon();
                    Ok(Stmt::Say(exprs))
                }
            }
	    TokenKind::Use => {
                self.advance(); // consume 'use'
                let module = if let TokenKind::Ident(n) = self.peek() {
                    let n = n.clone();
                    self.advance();
                    n
                } else {
                    return Err(ParseError {
                        message: "Expected module name after 'use'".into(),
                        span: None,
                    });
                };
                self.maybe_consume_semicolon();
                Ok(Stmt::Use(module))
            }
            TokenKind::Fn => self.parse_fn_stmt(),
            TokenKind::Try => self.parse_try_stmt(),
	    TokenKind::Match => self.parse_match_stmt(),
            TokenKind::Die => self.parse_die_stmt(),
            TokenKind::Throw => self.parse_throw_stmt(),
            TokenKind::LBrace => self.parse_block_stmt(),
            TokenKind::Semicolon => {
                self.advance(); // empty statement
                Ok(Stmt::Expr(Box::new(Expr::Null)))
            }
                        // Variable or expression starting with identifier/sigil
            TokenKind::Ident(_) | TokenKind::Dollar | TokenKind::At | TokenKind::Ampersand => {
                let expr = self.parse_expression()?;

                if self.peek() == TokenKind::Equal {
                    self.advance(); // consume '='
                    let value = Box::new(self.parse_expression()?);
                    self.maybe_consume_semicolon();

                    if let Expr::Var { name, .. } = expr {
                        return Ok(Stmt::Assign { name, expr: value });
                    }

                    return Err(ParseError {
                        message: "Invalid assignment target".into(),
                        span: None,
                    });
                }

                self.maybe_consume_semicolon();
                Ok(Stmt::Expr(Box::new(expr)))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.maybe_consume_semicolon();
                Ok(Stmt::Expr(Box::new(expr)))
            }
        }
    }

    /// Optionally consume a semicolon; not all statement types require one
    fn maybe_consume_semicolon(&mut self) {
        if self.peek() == TokenKind::Semicolon {
            self.advance();
        }
    }

    // ─── Try/Catch/Finally ────────────────────────────────────────────────

    fn parse_try_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'try'

        // Parse try body
        self.expect(TokenKind::LBrace)?;
        let try_block = self.parse_block()?; // consumes '}'

        // Parse optional catch
        let mut catch_var: Option<String> = None;
        let mut catch_block: Option<Vec<Stmt>> = None;

        if self.peek() == TokenKind::Catch {
            self.advance(); // consume 'catch'

            // Optional variable:   catch e { ... }
            // Or with alias:       catch e as err { ... }
            if let TokenKind::Ident(name) = self.peek() {
                let first_name = name.clone();
                self.advance();

                // Check for 'as' alias — use the alias as the variable name
                if self.peek() == TokenKind::As {
                    self.advance(); // consume 'as'
                    if let TokenKind::Ident(alias) = self.peek() {
                        catch_var = Some(alias.clone());
                        self.advance();
                    } else {
                        return Err(ParseError {
                            message: "Expected variable name after 'as'".into(),
                            span: None,
                        });
                    }
                } else {
                    catch_var = Some(first_name);
                }
            }

            self.expect(TokenKind::LBrace)?;
            catch_block = Some(self.parse_block()?);
        }

        // Parse optional finally
        let mut finally_block: Option<Vec<Stmt>> = None;

        if self.peek() == TokenKind::Finally {
            self.advance(); // consume 'finally'
            self.expect(TokenKind::LBrace)?;
            finally_block = Some(self.parse_block()?);
        }

        // Must have at least catch or finally
        if catch_block.is_none() && finally_block.is_none() {
            return Err(ParseError {
                message: "Expected 'catch' or 'finally' after 'try' block".into(),
                span: None,
            });
        }

        Ok(Stmt::TryCatch {
            try_block,
            catch_var,
            catch_block,
            finally_block,
        })
    }

    // ─── Match statement ───────────────────────────────────────────────────

    fn parse_match_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'match'

        let value = Box::new(self.parse_expression()?);
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::EOF {
            // ── Parse pattern ─────────────────────────────────────
            let pattern = match self.peek() {
                TokenKind::Ident(ref s) if s == "_" => {
                    self.advance();
                    MatchPattern::Wildcard
                }
                TokenKind::Int(s) => {
                    let i: i64 = s.parse().unwrap_or(0);
                    self.advance();
                    MatchPattern::Int(i)
                }
                TokenKind::StringLit(s) | TokenKind::SingleString(s) => {
                    let inner = &s[1..s.len() - 1];
                    self.advance();
                    MatchPattern::Str(inner.to_string())
                }
                TokenKind::True => {
                    self.advance();
                    MatchPattern::Bool(true)
                }
                TokenKind::False => {
                    self.advance();
                    MatchPattern::Bool(false)
                }
                TokenKind::Ident(s) => {
                    let name = s.clone();
                    self.advance();
                    MatchPattern::Var(name)
                }
                _ => {
                    return Err(ParseError {
                        message: format!("Invalid match pattern: {:?}", self.peek()),
                        span: None,
                    });
                }
            };

            // ── Optional 'where' guard ────────────────────────────
            let guard = if self.peek() == TokenKind::Where {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.expect(TokenKind::FatComma)?; // =>

            // ── Arm body (block or single expression) ─────────────
            let body = if self.peek() == TokenKind::LBrace {
                self.advance();
                self.parse_block()?
            } else {
                let expr = self.parse_expression()?;
                vec![Stmt::Return(Some(Box::new(expr)))]
            };

            arms.push(MatchArm { pattern, guard, body });

            // Optional trailing comma
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Stmt::Match { value, arms })
    }

    
    // ─── Block statement (standalone { ... } block) ──────────────────────

    fn parse_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume '{'
        let mut stmts = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::EOF {
            stmts.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Stmt::Block(stmts))
    }

    /// Parse statements until '}', consuming the closing brace.
    /// Used by try/catch/finally, fn bodies, if/else bodies, etc.
    pub fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::EOF {
            stmts.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    // ─── Let statement ───────────────────────────────────────────────────

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'let'

        let sigil = self.parse_sigil();

        let name = if let TokenKind::Ident(n) = self.peek() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err(ParseError {
                message: "Expected variable name after 'let'".into(),
                span: None,
            });
        };

        let mut expr: Option<Box<Expr>> = None;
        if self.peek() == TokenKind::Equal {
            self.advance();
            expr = Some(Box::new(self.parse_expression()?));
        }

        self.maybe_consume_semicolon();

        Ok(Stmt::Let {
            name,
            sigil,
            expr,
            mutable: true,
        })
    }

    // ─── Const statement ─────────────────────────────────────────────────

    fn parse_const_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'const'

        let sigil = self.parse_sigil();

        let name = if let TokenKind::Ident(n) = self.peek() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err(ParseError {
                message: "Expected variable name after 'const'".into(),
                span: None,
            });
        };

        self.expect(TokenKind::Equal)?;
        let expr = Box::new(self.parse_expression()?);

        self.maybe_consume_semicolon();

        Ok(Stmt::Const { name, sigil, expr })
    }

    // ─── If statement ────────────────────────────────────────────────────

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'if'

        let condition = Box::new(self.parse_expression()?);

        self.expect(TokenKind::LBrace)?;
        let then_block = self.parse_block()?;

        let mut else_block: Option<Vec<Stmt>> = None;
        if self.peek() == TokenKind::Else {
            self.advance(); // consume 'else'

            if self.peek() == TokenKind::If {
                // `else if` becomes a nested If in the else block
                else_block = Some(vec![self.parse_if_stmt()?]);
            } else {
                self.expect(TokenKind::LBrace)?;
                else_block = Some(self.parse_block()?);
            }
        }

        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    // ─── While statement ─────────────────────────────────────────────────

    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'while'

        let condition = Box::new(self.parse_expression()?);

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;

        Ok(Stmt::While { condition, body })
    }

    // ─── For statement ───────────────────────────────────────────────────

    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'for'

        let variable = if let TokenKind::Ident(n) = self.peek() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err(ParseError {
                message: "Expected variable name after 'for'".into(),
                span: None,
            });
        };

        self.expect(TokenKind::In)?;

        let iterable = Box::new(self.parse_expression()?);

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;

        Ok(Stmt::For {
            variable,
            iterable,
            body,
        })
    }

    // ─── Return statement ────────────────────────────────────────────────

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'return'

        let expr = if self.peek() == TokenKind::Semicolon
            || self.peek() == TokenKind::RBrace
            || self.peek() == TokenKind::EOF
        {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        self.maybe_consume_semicolon();

        Ok(Stmt::Return(expr))
    }

    // ─── Break / Continue ────────────────────────────────────────────────

    fn parse_break_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'break'
        self.maybe_consume_semicolon();
        Ok(Stmt::Break)
    }

    fn parse_continue_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'continue'
        self.maybe_consume_semicolon();
        Ok(Stmt::Continue)
    }

    // ─── Die / Throw statements ────────────────────────────

    fn parse_die_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'die'
        let expr = Box::new(self.parse_expression()?);
        self.maybe_consume_semicolon();
        Ok(Stmt::Die(expr))
    }

    fn parse_throw_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'throw'
        let expr = Box::new(self.parse_expression()?);
        self.maybe_consume_semicolon();
        Ok(Stmt::Throw(expr))
    }

    // ─── Fn statement ────────────────────────────────────────────────────

    fn parse_fn_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'fn'

        let name = if let TokenKind::Ident(n) = self.peek() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err(ParseError {
                message: "Expected function name after 'fn'".into(),
                span: None,
            });
        };

        self.expect(TokenKind::LParen)?;

        let params = self.parse_params()?;

        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;

        Ok(Stmt::Fn { name, params, body })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.peek() == TokenKind::RParen {
            return Ok(params);
        }

        loop {
            let sigil = self.parse_sigil();
            let name = if let TokenKind::Ident(n) = self.peek() {
                let n = n.clone();
                self.advance();
                n
            } else {
                return Err(ParseError {
                    message: "Expected parameter name".into(),
                    span: None,
                });
            };

            let mut type_hint = None;
            let mut default = None;

            if self.peek() == TokenKind::Colon {
                self.advance();
                if let TokenKind::Ident(t) = self.peek() {
                    type_hint = Some(t.clone());
                    self.advance();
                }
            }

            if self.peek() == TokenKind::Equal {
                self.advance();
                default = Some(self.parse_expression()?);
            }

            params.push(Param {
                name,
                sigil,
                type_hint,
                default,
            });

            if self.peek() == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        Ok(params)
    }

    // ─── Sigil helper ────────────────────────────────────────────────────

    fn parse_sigil(&mut self) -> VarType {
        match self.peek() {
            TokenKind::Dollar => {
                self.advance();
                VarType::Scalar
            }
            TokenKind::At => {
                self.advance();
                VarType::Array
            }
            TokenKind::Ampersand => {
                self.advance();
                VarType::Ref
            }
            TokenKind::Backslash => {
                self.advance();
                VarType::Ref
            }
            _ => VarType::Scalar, // default
        }
    }

    // ─── Expression parsing (Pratt parser) ───────────────────────────────

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expr, ParseError> {
        let condition = self.parse_binary(0)?;
        if self.peek() != TokenKind::Question {
            return Ok(condition);
        }
        self.advance(); // consume '?'
        let then_expr = Box::new(self.parse_expression()?);
        self.expect(TokenKind::Colon)?;
        // Right-associative: parse ternary recursively for else branch
        let else_expr = Box::new(self.parse_ternary()?);
        Ok(Expr::Ternary {
            condition: Box::new(condition),
            then_expr,
            else_expr,
        })
    }


    fn parse_binary(&mut self, min_prec: i32) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;

        loop {
            let token = self.peek();
            let prec = self.token_precedence(&token);

            if prec < min_prec {
                break;
            }

            let op = self.parse_binary_op(&token)?;
            self.advance(); // consume the operator
            let next_prec = if op == BinaryOp::Pow { prec } else { prec + 1 };
            let right = self.parse_binary(next_prec)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn token_precedence(&self, token: &TokenKind) -> i32 {
        match token {
            // Logical
            TokenKind::Or => 1,
            TokenKind::And => 2,
            // Bitwise
            TokenKind::Pipe => 3,
            TokenKind::Caret => 4,
            TokenKind::Ampersand => 5,
            TokenKind::ShiftLeft | TokenKind::ShiftRight => 6,
            // Equality / match
            TokenKind::EqualEqual
            | TokenKind::NotEqual
            | TokenKind::Spaceship
            | TokenKind::MatchOp
            | TokenKind::NotMatchOp => 7,
            // Comparison
            TokenKind::Less
            | TokenKind::Greater
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual => 8,
            // Additive
            TokenKind::Plus | TokenKind::Minus => 9,
            TokenKind::Dot => 10,
            // Multiplicative / repeat
            TokenKind::Star | TokenKind::Slash | TokenKind::Modulo => 11,
            TokenKind::Ident(s) if s == "x" => 11,
            // Power
            TokenKind::Pow => 12,
            _ => -1,
        }
    }

    fn parse_binary_op(&self, token: &TokenKind) -> Result<BinaryOp, ParseError> {
        match token {
            TokenKind::Plus => Ok(BinaryOp::Add),
            TokenKind::Minus => Ok(BinaryOp::Sub),
            TokenKind::Star => Ok(BinaryOp::Mul),
            TokenKind::Slash => Ok(BinaryOp::Div),
            TokenKind::Modulo => Ok(BinaryOp::Mod),
            TokenKind::Pow => Ok(BinaryOp::Pow),
            TokenKind::EqualEqual => Ok(BinaryOp::Eq),
            TokenKind::NotEqual => Ok(BinaryOp::Ne),
            TokenKind::Less => Ok(BinaryOp::Lt),
            TokenKind::Greater => Ok(BinaryOp::Gt),
            TokenKind::LessEqual => Ok(BinaryOp::Le),
            TokenKind::GreaterEqual => Ok(BinaryOp::Ge),
            TokenKind::Spaceship => Ok(BinaryOp::Cmp),
            TokenKind::And => Ok(BinaryOp::And),
            TokenKind::Or => Ok(BinaryOp::Or),
            TokenKind::Pipe => Ok(BinaryOp::BitOr),
	    TokenKind::Caret => Ok(BinaryOp::BitXor),
            TokenKind::Ampersand => Ok(BinaryOp::BitAnd), 
            TokenKind::ShiftLeft => Ok(BinaryOp::ShiftLeft),
            TokenKind::ShiftRight => Ok(BinaryOp::ShiftRight),
            TokenKind::MatchOp => Ok(BinaryOp::Match),
	    TokenKind::Dot => Ok(BinaryOp::Concat),
            TokenKind::Ident(s) if s == "x" => Ok(BinaryOp::Repeat),
            TokenKind::NotMatchOp => Ok(BinaryOp::NotMatch),
            _ => Err(ParseError {
                message: format!("Unexpected binary operator: {:?}", token),
                span: None,
            }),
        }
    }

    // ─── Prefix / Primary expressions ────────────────────────────────────

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek();

        match token {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Backslash => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Ref(Box::new(expr)))
            }
            TokenKind::Caret => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Deref(Box::new(expr)))
            }
	    TokenKind::Tilde => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek();

        match token {
            TokenKind::Int(s) => {
                self.advance();
                let i: i64 = s.parse().map_err(|_| ParseError {
                    message: format!("Invalid integer: {}", s),
                    span: None,
                })?;
                Ok(Expr::Int(i))
            }
            TokenKind::Float(s) => {
                self.advance();
                let f: f64 = s.parse().map_err(|_| ParseError {
                    message: format!("Invalid float: {}", s),
                    span: None,
                })?;
                Ok(Expr::Float(f))
            }
            TokenKind::StringLit(s) | TokenKind::SingleString(s) => {
                self.advance();
                // Strip surrounding quotes
                let inner = &s[1..s.len() - 1];
                Ok(Expr::Str(inner.to_string()))
            }
            TokenKind::Regex(s) | TokenKind::QuotedRegex(s) => {
                self.advance();
                Ok(Expr::Regex(s.clone()))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                self.parse_postfix(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if self.peek() != TokenKind::RBracket {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::Array(elements))
            }
            TokenKind::LBrace => {
                self.advance();
                let mut pairs = Vec::new();
                if self.peek() != TokenKind::RBrace {
                    loop {
                        let key = self.parse_expression()?;

                        // Convert bare identifiers in hash keys to string literals
                        // { name => "value" } → "name" as key, not variable lookup
                        let key = match key {
                            Expr::Var { name, sigil: None } => Expr::Str(name),
                            other => other,
                        };

                        if self.peek() == TokenKind::FatComma {
                            self.advance(); // consume '=>'
                            let value = self.parse_expression()?;
                            pairs.push((key, value));
                        } else if self.peek() == TokenKind::Comma
                            || self.peek() == TokenKind::RBrace
                        {
                            // Bare key: { name, age } → shorthand for { name => name, age => age }
                            let value = key.clone();
                            pairs.push((key, value));
                        } else {
                            return Err(ParseError {
                                message: "Expected '=>' or ',' in hash literal".into(),
                                span: None,
                            });
                        }

                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Hash(pairs))
            }
            TokenKind::Pipe => {
                self.advance(); // consume '|'
                let mut params = Vec::new();
                if self.peek() != TokenKind::Pipe {
                    loop {
                        if let TokenKind::Ident(n) = self.peek() {
                            params.push(n.clone());
                            self.advance();
                        } else {
                            return Err(ParseError {
                                message: "Expected parameter name in lambda".into(),
                                span: None,
                            });
                        }
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::Pipe)?;
                self.expect(TokenKind::FatComma)?; // =>

                if self.peek() == TokenKind::LBrace {
                    // Block body: |x| => { return x * x; }
                    self.advance();
                    let body = self.parse_block()?;
                    Ok(Expr::Lambda { params, body })
                } else {
                    // Expression body: |x| => x * 2
                    let expr = self.parse_expression()?;
                    let body = vec![Stmt::Return(Some(Box::new(expr)))];
                    Ok(Expr::Lambda { params, body })
                }
            }

            // ─── Plain identifier ────────────────────────────────────
            TokenKind::Ident(s) => {
                self.advance();
                let expr = Expr::Var {
                    name: s,
                    sigil: None,
                };
                // Use parse_postfix which converts .method(args) → Expr::Method
                self.parse_postfix(expr)
            }

            // ─── Sigil-prefixed variables: $scalar, @array, &ref ─────
            TokenKind::Dollar | TokenKind::At | TokenKind::Ampersand => {
                let sigil = self.parse_sigil();
                let name = if let TokenKind::Ident(n) = self.peek() {
                    let n = n.clone();
                    self.advance();
                    n
                } else {
                    return Err(ParseError {
                        message: "Expected variable name after sigil".into(),
                        span: None,
                    });
                };

                let expr = Expr::Var {
                    name,
                    sigil: Some(sigil),
                };
                // Use parse_postfix which converts .method(args) → Expr::Method
                self.parse_postfix(expr)
            }

            _ => Err(ParseError {
                message: format!("Unexpected token: {:?}", token),
                span: None,
            }),
        }
    }

    // ─── Postfix parsing (index, call, method call, field access) ────────

    /// Postfix chain: `expr[idx]`, `expr(args)`, `expr.field`, `expr.method(args)`.
    /// Converts `.method(args)` into `Expr::Method { object, method, args }`
    /// so the interpreter dispatches array methods (push, pop, map, filter…) correctly.
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        loop {
            match self.peek() {
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::Index {
                        collection: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                TokenKind::LParen => {
                    expr = self.parse_call(expr)?;
                }
                TokenKind::Dot => {
                    self.advance(); // consume '.'

                    let method = if let TokenKind::Ident(n) = self.peek() {
                        let n = n.clone();
                        self.advance();
                        n
                    } else {
                        return Err(ParseError {
                            message: "Expected field name after '.'".into(),
                            span: None,
                        });
                    };

                    // If followed by '(' → method call → Expr::Method
                    if self.peek() == TokenKind::LParen {
                        self.advance(); // consume '('
                        let mut args = Vec::new();
                        if self.peek() != TokenKind::RParen {
                            loop {
                                args.push(self.parse_expression()?);
                                if self.peek() == TokenKind::Comma {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                        self.expect(TokenKind::RParen)?;
                        expr = Expr::Method {
                            object: Box::new(expr),
                            method,
                            args,
                        };
                    } else {
                        // Plain field access (no parens) → Expr::FieldAccess
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field: method,
                        };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Parse function call:  expr(args)
    fn parse_call(&mut self, func: Expr) -> Result<Expr, ParseError> {
        self.advance(); // consume '('

        let mut args = Vec::new();
        let mut named_args = Vec::new();

        if self.peek() != TokenKind::RParen {
            loop {
                let arg = self.parse_expression()?;

                // Check for named argument:  name => value
                if self.peek() == TokenKind::FatComma {
                    if let Expr::Var { name, .. } = &arg {
                        let n = name.clone();
                        self.advance(); // consume '=>'
                        let value = self.parse_expression()?;
                        named_args.push((n, value));
                    } else {
                        return Err(ParseError {
                            message: "Expected variable name before '=>' in named argument".into(),
                            span: None,
                        });
                    }
                } else {
                    args.push(arg);
                }

                if self.peek() == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;

        Ok(Expr::Call {
            func: Box::new(func),
            args,
            named_args,
        })
    }

    // ─── Expect helper ───────────────────────────────────────────────────

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
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
                span: None,
            })
        }
    }
}
