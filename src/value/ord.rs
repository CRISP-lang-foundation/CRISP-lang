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

//! Porovnávanie hodnôt (Rust-style cmp, PartialOrd)

use crate::eval::RuntimeError;
use crate::value::Value;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum CmpResult {
    Less,
    Equal,
    Greater,
}

impl From<Ordering> for CmpResult {
    fn from(ord: Ordering) -> Self {
        match ord {
            Ordering::Less => CmpResult::Less,
            Ordering::Equal => CmpResult::Equal,
            Ordering::Greater => CmpResult::Greater,
        }
    }
}

impl CmpResult {
    pub fn to_i64(&self) -> i64 {
        match self {
            CmpResult::Less => -1,
            CmpResult::Equal => 0,
            CmpResult::Greater => 1,
        }
    }

    pub fn to_bool(&self) -> bool {
        matches!(self, CmpResult::Greater | CmpResult::Equal)
    }
}

impl Value {
    pub fn cmp(&self, other: &Value) -> Result<CmpResult, RuntimeError> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(a.cmp(b).into()),
            (Value::Float(a), Value::Float(b)) => match a.partial_cmp(b) {
                Some(ord) => Ok(ord.into()),
                None => Err(RuntimeError::CannotCompare),
            },
            (Value::Str(a), Value::Str(b)) => Ok(a.cmp(b).into()),
            (Value::Bool(a), Value::Bool(b)) => Ok(a.cmp(b).into()),
            (Value::Array(a), Value::Array(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                match a.len().cmp(&b.len()) {
                    Ordering::Equal => {
                        for (x, y) in a.iter().zip(b.iter()) {
                            match x.cmp(y)? {
                                CmpResult::Equal => continue,
                                other => return Ok(other),
                            }
                        }
                        Ok(CmpResult::Equal)
                    }
                    ord => Ok(ord.into()),
                }
            }
            (Value::Hash(a), Value::Hash(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                Ok(a.len().cmp(&b.len()).into())
            }
            (Value::Null, Value::Null) => Ok(CmpResult::Equal),
            _ => Err(RuntimeError::CannotCompare),
        }
    }

    // Perl-style spaceship operator
    pub fn spaceship(&self, other: &Value) -> Result<i64, RuntimeError> {
        match self.cmp(other) {
            Ok(result) => Ok(result.to_i64()),
            Err(_) => {
                // Fallback - Perl-style konverzia
                let a_num = self.as_number();
                let b_num = other.as_number();
                if let (Some(a), Some(b)) = (a_num, b_num) {
                    return Ok(match a.partial_cmp(&b) {
                        Some(ord) => ord as i64,
                        None => 0,
                    });
                }

                let a_str = self.as_str();
                let b_str = other.as_str();
                Ok(match a_str.cmp(&b_str) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                })
            }
        }
    }
}
