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

//! Number literal processing for CRISP lexer
//!
//! This module handles number literal parsing including:
//! - Decimal integers and floats
//! - Octal literals (0o644)
//! - Hexadecimal literals (0x1A3F)
//! - Binary literals (0b1010)
//! - Numeric separators (1_000_000)

use crate::lexer::TokenKind;
use crate::lexer::ParseResult;

/// Parse a number literal with optional prefix
///
/// # Examples
/// ```
/// use crisp::lexer::numbers::parse_number;
///
/// let result = parse_number("42");
/// assert!(matches!(result, Ok(TokenKind::Int(42))));
///
/// let result = parse_number("0o644");
/// assert!(matches!(result, Ok(TokenKind::Int(420))));
///
/// let result = parse_number("0x1A3F");
/// assert!(matches!(result, Ok(TokenKind::Int(6719))));
///
/// let result = parse_number("0b1010");
/// assert!(matches!(result, Ok(TokenKind::Int(10))));
/// ```
pub fn parse_number(input: &str) -> ParseResult {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut has_dot = false;
    let mut has_exponent = false;
    
    // Skip optional sign
    if i < chars.len() && (chars[i] == '-' || chars[i] == '+') {
        i += 1;
    }
    
    // Read integer part
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    
    // Read fractional part
    if i < chars.len() && chars[i] == '.' {
        has_dot = true;
        i += 1;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
    }
    
    // Read exponent (scientific notation)
    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
        has_exponent = true;
        i += 1;
        if i < chars.len() && (chars[i] == '-' || chars[i] == '+') {
            i += 1;
        }
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
    }
    
    let num_str = &input[0..i];
    
    if has_dot || has_exponent {
        // Parse as floating-point number
        if let Ok(num) = num_str.parse::<f64>() {
            return Ok(TokenKind::Float(num.to_string()));
        }
    } else {
        // Parse as integer
        if let Ok(num) = num_str.parse::<i64>() {
            return Ok(TokenKind::Int(num.to_string()));
        }
    }
    
    // Fallback - return the original string
    if has_dot || has_exponent {
        Ok(TokenKind::Float(num_str.to_string()))
    } else {
        Ok(TokenKind::Int(num_str.to_string()))
    }
}

/// Check if a string is a number with a prefix
pub fn has_number_prefix(s: &str) -> bool {
    s.starts_with("0o")
        || s.starts_with("0O")
        || s.starts_with("0x")
        || s.starts_with("0X")
        || s.starts_with("0b")
        || s.starts_with("0B")
}

/// Check if a string is a valid number literal (for quick validation)
pub fn is_number_literal(s: &str) -> bool {
    // Remove underscores for validation
    let cleaned: String = s.chars().filter(|&c| c != '_').collect();

    // Check prefixes
    if cleaned.starts_with("0o") || cleaned.starts_with("0O") {
        return cleaned[2..].chars().all(|c| c.is_digit(8));
    }
    if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
        return cleaned[2..].chars().all(|c| c.is_digit(16));
    }
    if cleaned.starts_with("0b") || cleaned.starts_with("0B") {
        return cleaned[2..].chars().all(|c| c.is_digit(2));
    }

    // Check if it's a float
    if cleaned.contains('.') {
        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.len() != 2 {
            return false;
        }
        return parts[0].chars().all(|c| c.is_digit(10))
            && parts[1].chars().all(|c| c.is_digit(10));
    }

    // Integer
    cleaned.chars().all(|c| c.is_digit(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        let result = parse_number("42").unwrap();
        assert!(matches!(result, TokenKind::Int(42)));

        let result = parse_number("-42").unwrap();
        assert!(matches!(result, TokenKind::Int(-42)));

        let result = parse_number("1_000_000").unwrap();
        assert!(matches!(result, TokenKind::Int(1000000)));
    }

    #[test]
    fn test_parse_octal() {
        let result = parse_number("0o644").unwrap();
        assert!(matches!(result, TokenKind::Int(420)));

        let result = parse_number("0O755").unwrap();
        assert!(matches!(result, TokenKind::Int(493)));
    }

    #[test]
    fn test_parse_hex() {
        let result = parse_number("0x1A3F").unwrap();
        assert!(matches!(result, TokenKind::Int(6719)));

        let result = parse_number("0XFF").unwrap();
        assert!(matches!(result, TokenKind::Int(255)));
    }

    #[test]
    fn test_parse_binary() {
        let result = parse_number("0b1010").unwrap();
        assert!(matches!(result, TokenKind::Int(10)));

        let result = parse_number("0B1111").unwrap();
        assert!(matches!(result, TokenKind::Int(15)));
    }

    #[test]
    fn test_parse_float() {
        let result = parse_number("3.14").unwrap();
        assert!(matches!(result, TokenKind::Float(3.14)));

        let result = parse_number("-2.5").unwrap();
        assert!(matches!(result, TokenKind::Float(-2.5)));
    }

    #[test]
    fn test_is_number_literal() {
        assert!(is_number_literal("42"));
        assert!(is_number_literal("0o644"));
        assert!(is_number_literal("0x1A3F"));
        assert!(is_number_literal("0b1010"));
        assert!(is_number_literal("3.14"));
        assert!(is_number_literal("1_000_000"));
        assert!(!is_number_literal("0o"));
        assert!(!is_number_literal("0x"));
        assert!(!is_number_literal("0b"));
        assert!(!is_number_literal("3.14.15"));
    }
}
