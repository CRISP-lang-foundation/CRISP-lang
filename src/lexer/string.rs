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

//! String processing utilities for CRISP lexer
//!
//! This module handles string literal processing including:
//! - Escape sequence handling (\n, \t, \", etc.)
//! - Character escaping and unescaping
//! - Unicode escape sequences (TODO)

use crate::lexer::TokenKind;
use crate::lexer::ParseResult;

/// Process escape sequences in a string literal
///
/// # Examples
/// ```
/// use crisp::lexer::process_escapes;
///
/// let s = "Hello\\nWorld";
/// let processed = process_escapes(s);
/// assert_eq!(processed, "Hello\nWorld");
/// ```
///
/// # Supported escape sequences
/// - `\n` - newline (LF)
/// - `\t` - tab
/// - `\r` - carriage return (CR)
/// - `\\` - backslash
/// - `\"` - double quote
/// - `\'` - single quote
/// - `\0` - null character
/// - `\u{xxxx}` - Unicode character
/// - `\xHH` - hexadecimal character
pub fn process_escapes(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut result = String::new();
    
    while i < chars.len() {
        let ch = chars[i];
        
        if ch == '\\' && i + 1 < chars.len() {
            // Escape sequence
            i += 1;
            match chars[i] {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                '0' => result.push('\0'),
                'x' => {
                    // Hex escape \xXX
                    i += 1;
                    if i + 1 < chars.len() {
                        let hex_str = format!("{}{}", chars[i], chars[i + 1]);
                        if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                            result.push(byte as char);
                            i += 1;
                        }
                    }
                }
                'u' => {
                    // Unicode escape \u{XXXX}
                    i += 1;
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let start = i;
                        while i < chars.len() && chars[i] != '}' {
                            i += 1;
                        }
                        if i < chars.len() && chars[i] == '}' {
                            let hex_str: String = chars[start..i].iter().collect();
                            if let Ok(codepoint) = u32::from_str_radix(&hex_str, 16) {
                                if let Some(c) = char::from_u32(codepoint) {
                                    result.push(c);
                                }
                            }
                        }
                    }
                }
                _ => {
                    result.push(chars[i]);
                }
            }
        } else {
            result.push(ch);
        }
        
        i += 1;
    }
    
    result
}

/// Parse a string literal and return a token
pub fn parse_string(input: &str) -> ParseResult {
    // Remove quotes and process escapes
    let unquoted = unquote(input);
    let processed = process_escapes(unquoted);
    Ok(TokenKind::StringLit(processed))
}

/// Remove quotes from a string literal
///
/// # Examples
/// ```
/// use crisp::lexer::unquote;
///
/// let s = "\"hello\"";
/// let unquoted = unquote(s);
/// assert_eq!(unquoted, "hello");
/// ```
pub fn unquote(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Check if a string contains escape sequences
pub fn has_escapes(s: &str) -> bool {
    s.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_escapes() {
        assert_eq!(process_escapes("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(process_escapes("Tab\\tHere"), "Tab\tHere");
        assert_eq!(process_escapes("Back\\\\slash"), "Back\\slash");
        assert_eq!(process_escapes("Quote\\\"Test"), "Quote\"Test");
        assert_eq!(process_escapes("Single\\'Test"), "Single'Test");
        assert_eq!(process_escapes("Null\\0Test"), "Null\0Test");
    }

    #[test]
    fn test_unquote() {
        assert_eq!(unquote("\"hello\""), "hello");
        assert_eq!(unquote("'world'"), "world");
        assert_eq!(unquote("plain"), "plain");
    }

    #[test]
    fn test_has_escapes() {
        assert!(has_escapes("Hello\\nWorld"));
        assert!(!has_escapes("Hello World"));
    }
}
