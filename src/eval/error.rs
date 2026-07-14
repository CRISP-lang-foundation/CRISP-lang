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

//! Error handling for the interpreter

use thiserror::Error;
use crate::value::Value;

#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Type mismatch")]
    TypeMismatch,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Argument error: {0}")]
    ArgumentError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Math error: {0}")]
    MathError(String),

    #[error("Recursion limit exceeded: {0}")]
    RecursionLimit(String),

    #[error("Cannot compare values")]
    CannotCompare,

    #[error("Pointer out of bounds")]
    PointerOutOfBounds,

    #[error("Invalid pointer")]
    InvalidPointer,

    /// Internal sentinel — not a user-facing error.
    /// Emitted by `return` so the function-body loop can extract the value
    /// and stop executing.
    #[error("Return signal")]
    ReturnSignal(Value),

    /// Internal sentinel — emitted by `break` inside a loop.
    #[error("Break signal")]
    BreakSignal,

    /// Internal sentinel — emitted by `continue` inside a loop.
    #[error("Continue signal")]
    ContinueSignal,

    /// User-triggered error via `throw()` or `die()`.
    /// The `{0}` format means catch blocks see the raw message
    /// without a "User error:" prefix.
    #[error("{0}")]
    UserError(String),

    /// Internal sentinel — emitted by `exit()`.
    /// Carries the process exit code. NOT catchable by try/catch.
    #[error("Exit signal ({0})")]
    ExitSignal(i32),
}

pub type EvalError = RuntimeError;

impl From<std::io::Error> for RuntimeError {
    fn from(err: std::io::Error) -> Self {
        RuntimeError::IOError(err.to_string())
    }
}
