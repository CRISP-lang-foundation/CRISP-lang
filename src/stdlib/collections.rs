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
use crate::value::CmpResult;
use crate::value::Value;

pub fn register(env: &mut Environment) {
    // map - transformácia
    env.define(
        "map",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "map needs function and array".into(),
                ));
            }

            let func = &args[0];
            let array = &args[1];

            let result = match array {
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    let mut results = Vec::new();
                    for elem in arr.iter() {
                        results.push(call_function(func, &[elem.clone()])?);
                    }
                    results
                }
                Value::Hash(hash) => {
                    let hash = hash.borrow();
                    let mut results = Vec::new();
                    for (key, value) in hash.iter() {
                        let args = vec![Value::Str(key.clone().into()), value.clone()];
                        results.push(call_function(func, &args)?);
                    }
                    results
                }
                _ => return Err(RuntimeError::TypeMismatch),
            };

            Ok(Value::Array(Rc::new(RefCell::new(result))))
        }),
    );

    // grep - filtrovanie
    env.define(
        "grep",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "grep needs function and array".into(),
                ));
            }

            let func = &args[0];
            let array = &args[1];

            let result = match array {
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    let mut results = Vec::new();
                    for elem in arr.iter() {
                        let result = call_function(func, &[elem.clone()])?;
                        if result.as_bool() {
                            results.push(elem.clone());
                        }
                    }
                    results
                }
                _ => return Err(RuntimeError::TypeMismatch),
            };

            Ok(Value::Array(Rc::new(RefCell::new(result))))
        }),
    );

    // sort - triedenie
    env.define(
        "sort",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("sort needs an array".into()));
            }

            match &args[0] {
                Value::Array(arr) => {
                    let mut arr = arr.borrow().clone();
                    if args.len() > 1 {
                        let func = &args[1];
                        arr.sort_by(|a, b| {
                            let result = call_function(func, &[a.clone(), b.clone()])
                                .unwrap_or(Value::Int(0));
                            match result {
                                Value::Int(i) => i.cmp(&0),
                                _ => std::cmp::Ordering::Equal,
                            }
                        });
                    } else {
                        arr.sort_by(|a, b| match a.cmp(b) {
                            Ok(CmpResult::Less) => std::cmp::Ordering::Less,
                            Ok(CmpResult::Equal) => std::cmp::Ordering::Equal,
                            Ok(CmpResult::Greater) => std::cmp::Ordering::Greater,
                            _ => std::cmp::Ordering::Equal,
                        });
                    }
                    Ok(Value::Array(Rc::new(RefCell::new(arr))))
                }
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );
}

fn call_function(func: &Value, args: &[Value]) -> Result<Value, RuntimeError> {
    match func {
        Value::NativeFn(f) => f(args),
        _ => Err(RuntimeError::TypeMismatch),
    }
}
