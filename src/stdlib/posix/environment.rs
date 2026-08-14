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
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;

pub fn register(env: &mut Environment) {
    env.define(
        "getenv",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "getenv needs a variable name".into(),
                ));
            }
            let name = args[0].as_str();
            match std::env::var(name) {
                Ok(value) => Ok(Value::Str(value.into())),
                Err(_) => Ok(Value::Null),
            }
        }),
    );

    env.define(
        "setenv",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "setenv needs name and value".into(),
                ));
            }
            let name = args[0].as_str();
            let value = args[1].as_str();
            unsafe { std::env::set_var(name, value) };
            Ok(Value::Bool(true))
        }),
    );

    env.define(
        "environ",
        Value::NativeFn(|_| {
            let mut hash = IndexMap::new();
            for (key, value) in std::env::vars() {
                hash.insert(key, Value::Str(value.into()));
            }
            Ok(Value::Hash(Rc::new(RefCell::new(hash))))
        }),
    );
}
