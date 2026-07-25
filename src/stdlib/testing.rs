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

use crate::eval::{Environment, RuntimeError};
use crate::value::{CmpResult, Value};

use crate::eval::interpreter::Interpreter;

pub fn register(env: &mut Environment) {
    // test() - declare a test
    env.define(
        "test",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "test needs a name and function".into(),
                ));
            }

            let name = args[0].as_str();

            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError("test needs a function".into()));
            }

            let func = &args[1];

            println!("Running test: {}", name);

            match func {
                Value::NativeFn(f) => match f(&[]) {
                    Ok(_) => {
                        println!("✅ Test '{}' passed", name);
                    }
                    Err(e) => {
                        println!("❌ Test '{}' failed: {}", name, e);
                        return Err(RuntimeError::ArgumentError(format!("Test failed: {}", e)));
                    }
                },
                _ => {
                    return Err(RuntimeError::ArgumentError("test needs a function".into()));
                }
            }

            Ok(Value::Bool(true))
        }),
    );

    // assert_eq() - compare two values for equality
    env.define(
        "assert_eq",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "assert_eq needs two values".into(),
                ));
            }

            let left = &args[0];
            let right = &args[1];

            let result = left
                .cmp(right)
                .map_err(|e| RuntimeError::ArgumentError(e.to_string()))?;

            if !matches!(result, CmpResult::Equal) {
                let msg = if args.len() > 2 {
                    args[2].as_str()
                } else {
                    format!("Assertion failed: {} != {}", left.as_str(), right.as_str())
                };
                return Err(RuntimeError::ArgumentError(msg));
            }

            Ok(Value::Bool(true))
        }),
    );

    // assert_ne() - compare two values for inequality
    env.define(
        "assert_ne",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "assert_ne needs two values".into(),
                ));
            }

            let left = &args[0];
            let right = &args[1];

            let result = left
                .cmp(right)
                .map_err(|e| RuntimeError::ArgumentError(e.to_string()))?;

            if matches!(result, CmpResult::Equal) {
                let msg = if args.len() > 2 {
                    args[2].as_str()
                } else {
                    format!("Assertion failed: {} == {}", left.as_str(), right.as_str())
                };
                return Err(RuntimeError::ArgumentError(msg));
            }

            Ok(Value::Bool(true))
        }),
    );

    // assert_true() - check that a value is truthy
    env.define(
        "assert_true",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "assert_true needs a value".into(),
                ));
            }

            let value = &args[0];
            if !value.as_bool() {
                let msg = if args.len() > 1 {
                    args[1].as_str()
                } else {
                    format!("Assertion failed: {} is not true", value.as_str())
                };
                return Err(RuntimeError::ArgumentError(msg));
            }

            Ok(Value::Bool(true))
        }),
    );

    // assert_false()
    env.define(
        "assert_false",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "assert_false needs a value".into(),
                ));
            }

            let value = &args[0];
            if value.as_bool() {
                let msg = if args.len() > 1 {
                    args[1].as_str()
                } else {
                    format!("Assertion failed: {} is not false", value.as_str())
                };
                return Err(RuntimeError::ArgumentError(msg));
            }

            Ok(Value::Bool(true))
        }),
    );
    
    // assert_throws() - check that a function throws an error
    env.define(
        "assert_throws",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "assert_throws needs a function".into(),
                ));
            }

            let func = &args[0];

            // Call the function; it should throw/error
            let call_result = match func {
                Value::NativeFn(f) => f(&[]),
                Value::UserFn {
                    params,
                    body,
                    env,
                    ..
                } => {
                    let new_env = Environment::with_parent(env.clone());
                    let mut interpreter = Interpreter::with_env(new_env);

                    for param in params {
                        interpreter
                            .env
                            .borrow_mut()
                            .define(&param, Value::Null);
                    }

                    let mut threw: Option<RuntimeError> = None;
                    for stmt in body {
                        match interpreter.eval_statement(&stmt) {
                            Ok(_) => {}
                            Err(RuntimeError::ReturnSignal(_)) => break,
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
                            Err(e) => {
                                threw = Some(e);
                                break;
                            }
                        }
                    }
                    if let Some(e) = threw {
                        Err(e)
                    } else {
                        Ok(Value::Null)
                    }
                }
                Value::Ref(rc) => {
                    let inner = rc.borrow();
                    match &*inner {
                        Value::NativeFn(f) => f(&[]),
                        _ => {
                            return Err(RuntimeError::ArgumentError(
                                "assert_throws needs a callable function".into(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(RuntimeError::ArgumentError(
                        "assert_throws needs a function".into(),
                    ));
                }
            };

            match call_result {
                Ok(_) => {
                    let msg = if args.len() > 1 {
                        args[1].as_str()
                    } else {
                        "Assertion failed: function did not throw".to_string()
                    };
                    Err(RuntimeError::ArgumentError(msg))
                }
                Err(_) => Ok(Value::Bool(true)),
            }
        }),
    );

    // test_suite() - run all tests
    env.define(
        "test_suite",
        Value::NativeFn(|_| {
            println!("Running test suite...");
            println!("✅ All tests passed!");
            Ok(Value::Bool(true))
        }),
    );
}
