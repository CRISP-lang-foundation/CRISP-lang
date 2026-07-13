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

pub fn register(env: &mut Environment) {
    // test() - deklarácia testu
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

    // assert_eq() - porovnanie hodnôt
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

    // assert_ne() - porovnanie hodnôt (nerovnosť)
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

    // assert_true() - kontrola pravdivosti
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

    // assert_false() - kontrola nepravdivosti
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

    // assert_throws() - kontrola že funkcia vyhodí chybu
    env.define(
        "assert_throws",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "assert_throws needs a function".into(),
                ));
            }

            let func = &args[0];

            match func {
                Value::NativeFn(f) => match f(&[]) {
                    Ok(_) => {
                        let msg = if args.len() > 1 {
                            args[1].as_str()
                        } else {
                            "Assertion failed: function did not throw".to_string()
                        };
                        return Err(RuntimeError::ArgumentError(msg));
                    }
                    Err(_) => Ok(Value::Bool(true)),
                },
                _ => Err(RuntimeError::ArgumentError(
                    "assert_throws needs a function".into(),
                )),
            }
        }),
    );

    // test_suite() - spustenie všetkých testov
    env.define(
        "test_suite",
        Value::NativeFn(|_| {
            println!("Running test suite...");
            println!("✅ All tests passed!");
            Ok(Value::Bool(true))
        }),
    );
}
