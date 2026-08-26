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

//! Token definitions for CRISP
//!
//! This module defines all the tokens used by the CRISP lexer.
//! Tokens are generated using the `logos` crate.

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip(r"#[^\n]*", allow_greedy = true))]
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
#[logos(skip(r"/\*([^*]|\*[^/])*\*/"))]
pub enum TokenKind {
    // Keywords – MUST be before Ident
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("use")]
    Use,
    #[token("print")]
    Print,
    #[token("say")]
    Say,
    #[token("die")]
    Die,
    #[token("throw")]
    Throw,
    #[token("warn")]
    Warn,
    #[token("assert")]
    Assert,
    #[token("where")]
    Where,
    #[token("in")]
    In,
    #[token("match")]
    Match,
    #[token("fn")]
    Fn,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("finally")]
    Finally,
    #[token("as")]
    As,

    // --- OOP keywords ---
    #[token("class")]
    Class,
    #[token("extends")]
    Extends,
    #[token("self")]
    SelfKeyword,
    #[token("super")]
    Super,
    #[token("static")]
    Static,

    // Sigils
    #[token("$")]
    Dollar,
    #[token("@")]
    At,
    #[token("%")] // Both modulo and hash sigil (context-dependent)
    Modulo,
    #[token("&")]
    Ampersand,
    #[token("\\")]
    Backslash,
    #[token("^")]
    Caret,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("**")]
    Pow,

    #[token("=")]
    Equal,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("<=>")]
    Spaceship,

    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,

    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    ModAssign,
    #[token("**=")]
    PowAssign,

    #[token(".")]
    Dot,

    #[token("=~")]
    MatchOp,
    #[token("!~")]
    NotMatchOp,
    #[token("tr")]
    Translate,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("=>")]
    FatComma,
    #[token("->")]
    Arrow,
    #[token("?")]
    Question,
    #[token("|")]
    Pipe,
    #[token("<<")]
    ShiftLeft,
    #[token(">>")]
    ShiftRight,
    #[token("~")]
    Tilde,

    // Regex literals - MUST come BEFORE Ident
    #[regex(r"m/[^/\n]*/", |lex| lex.slice().to_string())]
    Regex(String),

    // qr/pattern/ - quoted regex
    #[regex(r"qr/[^/\n]*/", |lex| lex.slice().to_string())]
    QuotedRegex(String),

    // Identifiers
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Integer literals — decimal, hex (0x/0X), octal (0o/0O), binary (0b/0B)
    #[regex(r"-?0[xX][0-9a-fA-F_]+|-?0[oO][0-7_]+|-?0[bB][01_]+|-?[0-9][0-9_]*", |lex| lex.slice().to_string())]
    Int(String),

    // Float literals
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    Float(String),

    // String literals - captured as raw strings, processed later
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    StringLit(String),
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| lex.slice().to_string())]
    SingleString(String),

    #[end]
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: std::ops::Range<usize>,
}

pub type Span = std::ops::Range<usize>;

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Token {
    /// Check if the token is a binary operator
    pub fn is_binary_op(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Modulo
                | TokenKind::Pow
                | TokenKind::EqualEqual
                | TokenKind::NotEqual
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::LessEqual
                | TokenKind::GreaterEqual
                | TokenKind::Spaceship
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::MatchOp
                | TokenKind::NotMatchOp
                | TokenKind::DotDot
                | TokenKind::Dot
        )
    }

    /// Get operator precedence for Pratt parser
    pub fn precedence(&self) -> i32 {
        match self.kind {
            TokenKind::Or => 1,
            TokenKind::And => 2,
            TokenKind::EqualEqual | TokenKind::NotEqual | TokenKind::Spaceship => 3,
            TokenKind::Less
            | TokenKind::Greater
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual => 4,
            TokenKind::Plus | TokenKind::Minus | TokenKind::Dot => 5,
            TokenKind::Star | TokenKind::Slash | TokenKind::Modulo => 6,
            TokenKind::Pow => 7,
            _ => 0,
        }
    }
}
