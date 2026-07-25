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
use crate::value::Value;

pub fn register(env: &mut Environment) {
    // length of string
    env.define(
        "length",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("length needs a string".into()));
            }

            let text = args[0].as_str();
            Ok(Value::Int(text.len() as i64))
        }),
    );

    // split string
    env.define(
        "split",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "split needs string and delimiter".into(),
                ));
            }

            let text = args[0].as_str();
            let delimiter = args[1].as_str();

            let parts: Vec<Value> = text
                .split(&delimiter)
                .map(|s| Value::Str(s.to_string().into()))
                .collect();

            Ok(Value::Array(Rc::new(RefCell::new(parts))))
        }),
    );

    // join strings
    env.define(
        "join",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "join needs delimiter and array".into(),
                ));
            }

            let delimiter = args[0].as_str();
            let array = &args[1];

            match array {
                Value::Array(arr) => {
                    let arr = arr.borrow();
                    let strings: Vec<String> = arr.iter().map(|v| v.as_str()).collect();
                    Ok(Value::Str(strings.join(&delimiter).into()))
                }
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );

    // trim the string
    env.define(
        "trim",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("trim needs a string".into()));
            }

            let text = args[0].as_str();
            Ok(Value::Str(text.trim().to_string().into()))
        }),
    );
}
