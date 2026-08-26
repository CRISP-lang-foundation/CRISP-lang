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

use crate::lexer::ParseResult;
use crate::lexer::TokenKind;

/// # Examples
/// ```
/// use crisp::lexer::parse_number;
/// use crisp::lexer::TokenKind;
///
/// let result = parse_number("42");
/// assert!(matches!(result, Ok(TokenKind::Int(ref s)) if s == "42"));
///
/// let result = parse_number("0o644");
/// assert!(matches!(result, Ok(TokenKind::Int(ref s)) if s == "420"));
///
/// let result = parse_number("0x1A3F");
/// assert!(matches!(result, Ok(TokenKind::Int(ref s)) if s == "6719"));
///
/// let result = parse_number("0b1010");
/// assert!(matches!(result, Ok(TokenKind::Int(ref s)) if s == "10"));
/// ```

pub fn parse_number(input: &str) -> ParseResult {
    // Remove numeric separators before parsing.
    let cleaned: String = input.chars().filter(|&c| c != '_').collect();
    let s = cleaned.as_str();

    let (sign, rest) = if let Some(r) = s.strip_prefix('-') {
        ("-", r)
    } else if let Some(r) = s.strip_prefix('+') {
        ("", r)
    } else {
        ("", s)
    };

    let lower = rest.to_ascii_lowercase();

    if let Some(hex) = lower.strip_prefix("0x") {
        if hex.is_empty() {
            return Err(format!("Invalid hexadecimal literal: {}", input));
        }
        let magnitude = i64::from_str_radix(hex, 16)
            .map_err(|e| format!("Invalid hexadecimal literal '{}': {}", input, e))?;
        let value = if sign == "-" { -magnitude } else { magnitude };
        return Ok(TokenKind::Int(value.to_string()));
    }

    if let Some(oct) = lower.strip_prefix("0o") {
        if oct.is_empty() {
            return Err(format!("Invalid octal literal: {}", input));
        }
        let magnitude = i64::from_str_radix(oct, 8)
            .map_err(|e| format!("Invalid octal literal '{}': {}", input, e))?;
        let value = if sign == "-" { -magnitude } else { magnitude };
        return Ok(TokenKind::Int(value.to_string()));
    }

    if let Some(bin) = lower.strip_prefix("0b") {
        if bin.is_empty() {
            return Err(format!("Invalid binary literal: {}", input));
        }
        let magnitude = i64::from_str_radix(bin, 2)
            .map_err(|e| format!("Invalid binary literal '{}': {}", input, e))?;
        let value = if sign == "-" { -magnitude } else { magnitude };
        return Ok(TokenKind::Int(value.to_string()));
    }

    // Decimal float or integer.
    if rest.contains('.') || rest.contains('e') || rest.contains('E') {
        let value = s
            .parse::<f64>()
            .map_err(|e| format!("Invalid float literal '{}': {}", input, e))?;
        Ok(TokenKind::Float(value.to_string()))
    } else {
        let value = s
            .parse::<i64>()
            .map_err(|e| format!("Invalid integer literal '{}': {}", input, e))?;
        Ok(TokenKind::Int(value.to_string()))
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
    let cleaned: String = s.chars().filter(|&c| c != '_').collect();
    let c = cleaned.as_str();

    if let Some(rest) = c.strip_prefix("0o").or_else(|| c.strip_prefix("0O")) {
        return !rest.is_empty() && rest.chars().all(|ch| ch.is_digit(8));
    }
    if let Some(rest) = c.strip_prefix("0x").or_else(|| c.strip_prefix("0X")) {
        return !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_hexdigit());
    }
    if let Some(rest) = c.strip_prefix("0b").or_else(|| c.strip_prefix("0B")) {
        return !rest.is_empty() && rest.chars().all(|ch| ch.is_digit(2));
    }

    if c.contains('.') {
        let parts: Vec<&str> = c.split('.').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return false;
        }
        return parts[0].chars().all(|d| d.is_ascii_digit())
            && parts[1].chars().all(|d| d.is_ascii_digit());
    }

    !c.is_empty() && c.chars().all(|d| d.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        let result = parse_number("42").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "42"));

        let result = parse_number("-42").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "-42"));

        let result = parse_number("1_000_000").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "1000000"));
    }

    #[test]
    fn test_parse_octal() {
        let result = parse_number("0o644").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "420"));

        let result = parse_number("0O755").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "493"));
    }

    #[test]
    fn test_parse_hex() {
        let result = parse_number("0x1A3F").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "6719"));

        let result = parse_number("0XFF").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "255"));
    }

    #[test]
    fn test_parse_binary() {
        let result = parse_number("0b1010").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "10"));

        let result = parse_number("0B1111").unwrap();
        assert!(matches!(result, TokenKind::Int(ref s) if s == "15"));
    }

    #[test]
    fn test_parse_float() {
        let result = parse_number("3.14").unwrap();
        assert!(matches!(result, TokenKind::Float(ref s) if s == "3.14"));

        let result = parse_number("-2.5").unwrap();
        assert!(matches!(result, TokenKind::Float(ref s) if s == "-2.5"));
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
