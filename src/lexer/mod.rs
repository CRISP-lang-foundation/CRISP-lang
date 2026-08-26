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

//! Lexer - converts source code into tokens
//!
//! This module is responsible for tokenizing CRISP source code.
//! It uses the `logos` crate for efficient token generation.

mod token;
mod string;
mod numbers;

pub use token::Span;
pub use token::{Token, TokenKind};
pub use string::process_escapes;
pub use string::unquote;
pub use string::parse_string;
pub use numbers::parse_number;
pub use numbers::has_number_prefix;

use crate::utils::diagnostics;
use logos::Logos;

pub type ParseResult = Result<TokenKind, String>;

#[derive(Debug, Clone)]
pub struct Lexer {
    tokens: Vec<Token>,
    source: String,
}

impl Lexer {
    /// Create a new lexer from source code
    ///
    /// # Arguments
    /// * `source` - The source code string to tokenize
    ///
    /// # Returns
    /// A new Lexer instance with tokenized source
    pub fn new(source: &str) -> Self {
        let source = source.to_string();
        let mut tokens = Vec::new();
        let mut lex = TokenKind::lexer(&source);
        
        if diagnostics::is_debug_mode() {
            diagnostics::log_debug(&format!("Processing: '{}'", source));
        }
        
        while let Some(Ok(tok)) = lex.next() {
            let span = lex.span();
            let slice = &source[span.clone()];
            
            if diagnostics::is_debug_mode() {
                diagnostics::log_debug(&format!("  Token: {:?} = '{}'", tok, slice));
            }
            
            // Process string literals to handle escape sequences
            let processed_tok = match tok {
                TokenKind::StringLit(s) => {
                    let processed = string::process_escapes(&s);
                    TokenKind::StringLit(processed)
                }
                TokenKind::SingleString(s) => {
                    let processed = string::process_escapes(&s);
                    TokenKind::StringLit(processed)
                }
                // parse hex/octal/binary integer literals
                TokenKind::Int(s) => {
                    let cleaned: String = s.chars().filter(|&c| c != '_').collect();
                    let parsed: i64 = if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
                        i64::from_str_radix(&cleaned[2..], 16).unwrap_or(0)
                    } else if cleaned.starts_with("0o") || cleaned.starts_with("0O") {
                        i64::from_str_radix(&cleaned[2..], 8).unwrap_or(0)
                    } else if cleaned.starts_with("0b") || cleaned.starts_with("0B") {
                        i64::from_str_radix(&cleaned[2..], 2).unwrap_or(0)
                    } else {
                        cleaned.parse::<i64>().unwrap_or(0)
                    };
                    TokenKind::Int(parsed.to_string())
                }
                _ => tok,
            };
            
            tokens.push(Token {
		kind: processed_tok,
		span,
            });
	}
        
        if diagnostics::is_debug_mode() {
            diagnostics::log_debug(&format!("Total {} tokens", tokens.len()));
        }
        
        Lexer { tokens, source }
    }
    
    /// Get the list of tokens
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }
    
    /// Get the source code
    pub fn source(&self) -> &str {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_with_escapes() {
	let input = r#""Hello\nWorld""#;
	let lexer = Lexer::new(input);
	let tokens = lexer.tokens();
	assert_eq!(tokens.len(), 1);
	if let TokenKind::StringLit(s) = &tokens[0].kind {
            assert_eq!(s, "\"Hello\nWorld\"");
	} else {
            panic!("Expected StringLit");
	}
    }
    
    #[test]
    fn test_string_with_tab() {
	let input = r#""Hello\tWorld""#;
	let lexer = Lexer::new(input);
	let tokens = lexer.tokens();
	assert_eq!(tokens.len(), 1);
	if let TokenKind::StringLit(s) = &tokens[0].kind {
            assert_eq!(s, "\"Hello\tWorld\"");
	} else {
            panic!("Expected StringLit");
	}
    }
    
    #[test]
    fn test_string_with_backslash() {
	let input = r#""Hello\World""#;
	let lexer = Lexer::new(input);
	let tokens = lexer.tokens();
	assert_eq!(tokens.len(), 1);
	if let TokenKind::StringLit(s) = &tokens[0].kind {
            // \W is not a recognized escape, so process_escapes drops the backslash
            assert_eq!(s, "\"HelloWorld\"");
	} else {
            panic!("Expected StringLit");
	}
    }

    #[test]
    fn test_number_literals() {
        let lexer = Lexer::new("42 3.14 0o644 0x1A3F 0b1010");
        let tokens = lexer.tokens();
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_number_with_underscores() {
        let lexer = Lexer::new("1_000_000");
        let tokens = lexer.tokens();
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn test_division_not_confused_with_regex() {
	let lexer = Lexer::new("10 / 2; 20 / 4;");
	let tokens = lexer.tokens();
	// Int, Slash, Int, Semicolon, Int, Slash, Int, Semicolon
	assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Slash)));
    }
}
