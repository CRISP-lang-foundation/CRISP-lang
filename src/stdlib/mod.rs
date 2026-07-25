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

//! Standard library for CRISP
//!
//! Registers all built-in functions and modules into the global environment.

use std::cell::RefCell;
use std::rc::Rc;

pub mod collections;
pub mod crypto;
pub mod filesystem;
pub mod io;
pub mod json;
pub mod math;
pub mod network;
pub mod process;
pub mod regex;
pub mod string;
pub mod testing;
pub mod time;

#[cfg(unix)]
pub mod posix;

pub use collections::*;
pub use crypto::*;
pub use filesystem::*;
pub use io::*;
pub use json::*;
pub use math::*;
pub use network::*;
pub use process::*;
pub use regex::*;
pub use string::*;
pub use testing::*;
pub use time::*;

use crate::eval::{Environment, RuntimeError};
use crate::value::Value;

pub fn register_all(env: &mut Environment) {
    // ── I/O (print, say, warn, readline, read, input + file ops) ──
    io::register(env);

    // ── Error/die/throw/assert (kept here — not I/O) ────────────────
    register_errors(env);

    // ── Modules ─────────────────────────────────────────────────────
    io::register(env);
    math::register(env);
    collections::register(env);
    json::register(env);
    regex::register(env);
    network::register(env);
    time::register(env);
    crypto::register(env);
    process::register(env);
    testing::register(env);
    filesystem::register(env);

    // ── Random ──────────────────────────────────────────────────────
    env.define(
        "rand",
        Value::NativeFn(|args| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42);

            // Simple xorshift64*
            let mut x = seed.wrapping_add(1);
            x ^= x.wrapping_shr(12);
            x ^= x.wrapping_shl(25);
            x ^= x.wrapping_shr(27);
            let r = x.wrapping_mul(0x2545F4914F6CDD1D);

            let max = args
                .first()
                .and_then(|a| {
                    if let Value::Int(n) = a {
                        Some(*n as u64)
                    } else {
                        None
                    }
                })
                .unwrap_or(u64::MAX);

            if max == 0 {
                return Ok(Value::Int(0));
            }
            Ok(Value::Int((r % max) as i64))
        }),
    );

    // ── Process control ─────────────────────────────────────────
    env.define(
        "exit",
        Value::NativeFn(|args| {
            let code = match args.first() {
                Some(Value::Int(n)) => *n as i32,
                Some(v) if v.as_number().is_some() => v.as_number().unwrap() as i32,
                Some(_) => 1,
                None => 0,
            };
            Err(crate::eval::RuntimeError::ExitSignal(code))
        }),
    );

    env.define(
        "_exit",
        Value::NativeFn(|args| {
            let code = match args.first() {
                Some(Value::Int(n)) => *n as i32,
                Some(v) if v.as_number().is_some() => v.as_number().unwrap() as i32,
                Some(_) => 1,
                None => 0,
            };
            std::process::exit(code);
        }),
    );
    
    register_builtins(env);
}

/// die / throw / assert — error handling, not I/O
fn register_errors(env: &mut Environment) {
    env.define(
        "die",
        Value::NativeFn(|args| {
            let msg = args.first().map_or("died".to_string(), |a| a.as_str());
            Err(RuntimeError::UserError(msg))
        }),
    );

    env.define(
        "throw",
        Value::NativeFn(|args| {
            let msg = args.first().map_or("thrown".to_string(), |a| a.as_str());
            Err(RuntimeError::UserError(msg))
        }),
    );

    env.define(
        "assert",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("assert needs a condition".into()));
            }
            if !args[0].as_bool() {
                let msg = args
                    .get(1)
                    .map_or("Assertion failed".to_string(), |a| a.as_str());
                return Err(RuntimeError::ArgumentError(msg));
            }
            Ok(Value::Null)
        }),
    );
}

/// General-purpose builtins: len, type, push, pop, map, filter
fn register_builtins(env: &mut Environment) {
    // len() — length of a string, array, or hash
    env.define(
        "len",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("len needs a value".into()));
            }
            match &args[0] {
                Value::Str(s) => Ok(Value::Int(s.len() as i64)),
                Value::Array(arr) => Ok(Value::Int(arr.borrow().len() as i64)),
                Value::Hash(hash) => Ok(Value::Int(hash.borrow().len() as i64)),
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );

    // type() — returns the type of a value
    env.define(
	"type",
	Value::NativeFn(|args| {
            if args.is_empty() {
		return Err(RuntimeError::ArgumentError("type needs a value".into()));
            }
            let type_name = match &args[0] {
		Value::Null => "null",
		Value::Bool(_) => "bool",
		Value::Int(_) => "int",
		Value::Float(_) => "float",
		Value::Str(_) => "string",
		Value::Array(_) => "array",
		Value::Hash(_) => "hash",
		Value::Ref(_) => "ref",
		Value::NativeFn(_) => "function",
		Value::UserFn { .. } => "function",
		Value::Class { .. } => "class",
		Value::Object { .. } => "object",
            };
            Ok(Value::Str(Rc::new(type_name.to_string())))
	}),
    );

    // int(value) → Int or Null
    env.define(
        "int",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("int needs a value".into()));
            }
            let s = args[0].as_str().trim().to_string();
            if s.is_empty() {
                return Ok(Value::Null);
            }
            if let Ok(i) = s.parse::<i64>() {
                Ok(Value::Int(i))
            } else {
                Ok(Value::Null)
            }
        }),
    );

    // float(value) → Float or Null
    env.define(
        "float",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("float needs a value".into()));
            }
            let s = args[0].as_str().trim().to_string();
            if s.is_empty() {
                return Ok(Value::Null);
            }
            if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Float(f))
            } else {
                Ok(Value::Null)
            }
        }),
    );

    // push(arr, elem1, elem2, ...)
    env.define(
        "push",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "push needs an array and at least one value".into(),
                ));
            }
            match &args[0] {
                Value::Array(a) => {
                    for val in &args[1..] {
                        a.borrow_mut().push(val.clone());
                    }
                    Ok(Value::Int(a.borrow().len() as i64))
                }
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );

    // pop(arr)
    env.define(
        "pop",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("pop needs an array".into()));
            }
            match &args[0] {
                Value::Array(a) => Ok(a.borrow_mut().pop().unwrap_or(Value::Null)),
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );

    // map(arr, callback)
    env.define(
        "map",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "map needs an array and a callback".into(),
                ));
            }
            let callback = &args[1];
            match &args[0] {
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    let mut result = Vec::new();
                    for elem in arr.iter() {
                        let mapped = call_value_direct(callback, &[elem.clone()])?;
                        result.push(mapped);
                    }
                    Ok(Value::Array(Rc::new(RefCell::new(result))))
                }
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );

    // filter(arr, callback)
    env.define(
        "filter",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "filter needs an array and a callback".into(),
                ));
            }
            let callback = &args[1];
            match &args[0] {
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    let mut result = Vec::new();
                    for elem in arr.iter() {
                        let keep = call_value_direct(callback, &[elem.clone()])?;
                        if keep.as_bool() {
                            result.push(elem.clone());
                        }
                    }
                    Ok(Value::Array(Rc::new(RefCell::new(result))))
                }
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );
}

/// Helper for builtins that need to invoke callbacks
fn call_value_direct(func: &Value, args: &[Value]) -> Result<Value, RuntimeError> {
    match func {
        Value::NativeFn(f) => f(args),
        Value::UserFn {
            params, body, env, ..
        } => {
            let new_env = Environment::with_parent(env.clone());
            let mut interpreter = crate::eval::Interpreter::with_env(new_env);
            for (i, param) in params.iter().enumerate() {
                if i < args.len() {
                    interpreter.env.borrow_mut().define(param, args[i].clone());
                } else {
                    interpreter.env.borrow_mut().define(param, Value::Null);
                }
            }
            let mut result = Value::Null;
            for stmt in body {
                match interpreter.eval_statement(stmt) {
                    Ok(val) => result = val,
                    Err(RuntimeError::ReturnSignal(v)) => {
                        result = v;
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(result)
        }
        _ => Err(RuntimeError::InvalidOperation(
            "Value is not callable".into(),
        )),
    }
}
