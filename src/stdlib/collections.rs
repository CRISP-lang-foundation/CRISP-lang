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

use std::cell::RefCell;
use std::rc::Rc;

use crate::eval::{Environment, RuntimeError};
use crate::eval::interpreter::Interpreter;
use crate::value::Value;

/// Unwrap Ref-wrapped values recursively to their inner value.
/// Returns a clone of the inner value.
fn unwrap_ref(value: &Value) -> Value {
    match value {
        Value::Ref(rc) => {
            let inner = rc.borrow();
            unwrap_ref(&inner)
        }
        other => other.clone(),
    }
}

/// Call any callable value (native or user-defined) with arguments.
fn call_function(func: &Value, args: &[Value]) -> Result<Value, RuntimeError> {
    match func {
        Value::NativeFn(f) => f(args),
        Value::UserFn {
            params,
            body,
            env,
            ..
        } => {
            let new_env = Environment::with_parent(env.clone());
            let mut interpreter = Interpreter::with_env(new_env);

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
                    Err(RuntimeError::BreakSignal) => {
                        return Err(RuntimeError::InvalidOperation(
                            "break outside loop".into(),
                        ));
                    }
                    Err(RuntimeError::ContinueSignal) => {
                        return Err(RuntimeError::InvalidOperation(
                            "continue outside loop".into(),
                        ));
                    }
                    Err(e @ RuntimeError::ExitSignal(_)) => return Err(e),
                    Err(e) => return Err(e),
                }
            }
            Ok(result)
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

/// map(array, function) — transform each element
pub fn map(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::ArgumentError(
            "map() requires an array and a function".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let func = &args[1];
    let arr = match &raw {
        Value::Array(a) => a.borrow(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    let mut result = Vec::new();
    for elem in arr.iter() {
        result.push(call_function(func, &[elem.clone()])?);
    }
    Ok(Value::Array(Rc::new(RefCell::new(result))))
}

/// grep(array, function) — filter elements matching predicate
pub fn grep(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::ArgumentError(
            "grep() requires an array and a function".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let func = &args[1];
    let arr = match &raw {
        Value::Array(a) => a.borrow(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    let mut result = Vec::new();
    for elem in arr.iter() {
        let keep = call_function(func, &[elem.clone()])?;
        if keep.as_bool() {
            result.push(elem.clone());
        }
    }
    Ok(Value::Array(Rc::new(RefCell::new(result))))
}

/// sort(array) — sort elements using default string comparison
/// sort(array, comparator) — sort using a custom comparator function
pub fn sort(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArgumentError(
            "sort() requires an array".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let arr = match &raw {
        Value::Array(a) => a.borrow(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    let mut sorted: Vec<Value> = arr.clone();

    if args.len() >= 2 {
        let cmp = &args[1];
        sorted.sort_by(|a, b| {
            let result = call_function(cmp, &[a.clone(), b.clone()]);
            match result {
                Ok(Value::Int(n)) if n < 0 => std::cmp::Ordering::Less,
                Ok(Value::Int(0)) => std::cmp::Ordering::Equal,
                Ok(Value::Int(n)) if n > 0 => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
    } else {
        sorted.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }

    Ok(Value::Array(Rc::new(RefCell::new(sorted))))
}

/// push(array, value, ...) — append one or more values to an array
pub fn push(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArgumentError(
            "push() requires an array".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let arr = match &raw {
        Value::Array(a) => a.clone(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    let mut arr = arr.borrow_mut();
    for val in &args[1..] {
        arr.push(val.clone());
    }
    Ok(Value::Int(arr.len() as i64))
}

/// pop(array) — remove and return the last element
pub fn pop(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArgumentError(
            "pop() requires an array".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let arr = match &raw {
        Value::Array(a) => a.clone(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    Ok(arr.borrow_mut().pop().unwrap_or(Value::Null))
}

/// shift(array) — remove and return the first element
pub fn shift(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArgumentError(
            "shift() requires an array".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let arr = match &raw {
        Value::Array(a) => a.clone(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    let mut arr = arr.borrow_mut();
    if arr.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(arr.remove(0))
    }
}

/// unshift(array, value, ...) — prepend one or more values to an array
pub fn unshift(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArgumentError(
            "unshift() requires an array".into(),
        ));
    }
    let raw = unwrap_ref(&args[0]);
    let arr = match &raw {
        Value::Array(a) => a.clone(),
        _ => return Err(RuntimeError::TypeMismatch),
    };
    let mut arr = arr.borrow_mut();
    for val in args[1..].iter().rev() {
        arr.insert(0, val.clone());
    }
    Ok(Value::Int(arr.len() as i64))
}

/// join(array, separator) — join array elements into a string
pub fn join(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::ArgumentError(
            "join() requires an array and a separator".into(),
        ));
    }

    let sep = args[1].as_str();

    // Use the same pattern as map and grep to get the array
    let raw = unwrap_ref(&args[0]);
    let arr = match &raw {
        Value::Array(a) => a.borrow(),
        _ => return Err(RuntimeError::TypeMismatch),
    };

    // Join the array elements into a string
    let joined: String = arr
        .iter()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(&sep);

    Ok(Value::Str(joined.into()))
}

/// Register all collection functions in the environment.
pub fn register(env: &mut Environment) {
    env.define("map", Value::NativeFn(map));
    env.define("grep", Value::NativeFn(grep));
    env.define("sort", Value::NativeFn(sort));
    env.define("push", Value::NativeFn(push));
    env.define("pop", Value::NativeFn(pop));
    env.define("shift", Value::NativeFn(shift));
    env.define("unshift", Value::NativeFn(unshift));
    env.define("join", Value::NativeFn(join));
}
