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
use regex::Regex;

pub fn register(env: &mut Environment) {
    // regex_match() - kontrola zhody
    env.define(
        "regex_match",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "regex_match needs text and pattern".into(),
                ));
            }

            let text = args[0].as_str();
            let pattern = args[1].as_str();

            let re = Regex::new(&pattern)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid regex: {}", e)))?;

            Ok(Value::Bool(re.is_match(&text)))
        }),
    );

    // regex_replace() - náhrada
    env.define(
        "regex_replace",
        Value::NativeFn(|args| {
            if args.len() < 3 {
                return Err(RuntimeError::ArgumentError(
                    "regex_replace needs text, pattern and replacement".into(),
                ));
            }

            let text = args[0].as_str();
            let pattern = args[1].as_str();
            let replacement = args[2].as_str();

            let re = Regex::new(&pattern)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid regex: {}", e)))?;

            let result = re.replace_all(&text, replacement);
            Ok(Value::Str(result.to_string().into()))
        }),
    );

    // regex_split() - rozdelenie podľa regex
    env.define(
        "regex_split",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "regex_split needs text and pattern".into(),
                ));
            }

            let text = args[0].as_str();
            let pattern = args[1].as_str();

            let re = Regex::new(&pattern)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid regex: {}", e)))?;

            let parts: Vec<Value> = re
                .split(&text)
                .map(|s| Value::Str(s.to_string().into()))
                .collect();

            Ok(Value::Array(Rc::new(RefCell::new(parts))))
        }),
    );

    // regex_find_all() - nájdenie všetkých výskytov
    env.define(
        "regex_find_all",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "regex_find_all needs text and pattern".into(),
                ));
            }

            let text = args[0].as_str();
            let pattern = args[1].as_str();

            let re = Regex::new(&pattern)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid regex: {}", e)))?;

            let matches: Vec<Value> = re
                .find_iter(&text)
                .map(|m| Value::Str(m.as_str().to_string().into()))
                .collect();

            Ok(Value::Array(Rc::new(RefCell::new(matches))))
        }),
    );

    // regex_capture() - zachytenie skupín
    env.define(
        "regex_capture",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "regex_capture needs text and pattern".into(),
                ));
            }

            let text = args[0].as_str();
            let pattern = args[1].as_str();

            let re = Regex::new(&pattern)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid regex: {}", e)))?;

            if let Some(caps) = re.captures(&text) {
                let mut result = Vec::new();
                for i in 0..caps.len() {
                    if let Some(m) = caps.get(i) {
                        result.push(Value::Str(m.as_str().to_string().into()));
                    }
                }
                Ok(Value::Array(Rc::new(RefCell::new(result))))
            } else {
                Ok(Value::Array(Rc::new(RefCell::new(Vec::new()))))
            }
        }),
    );
}
