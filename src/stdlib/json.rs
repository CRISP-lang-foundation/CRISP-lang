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
use std::collections::HashMap;
use std::rc::Rc;

use crate::eval::{Environment, RuntimeError};
use crate::value::Value;

use indexmap::IndexMap;
use serde_json::{Map, Number, Value as JsonValue};


pub fn register(env: &mut Environment) {
    // to_json() - conversion of value to JSON string
    env.define(
        "to_json",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("to_json needs a value".into()));
            }

            let value = &args[0];
            let json = value_to_json(value)?;
            Ok(Value::Str(json.into()))
        }),
    );

    // from_json() - conversion of JSON string to value
    env.define(
        "from_json",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "from_json needs a JSON string".into(),
                ));
            }

            let json_str = args[0].as_str();
            let value = json_to_value_from_str(&json_str)?;
            Ok(value)
        }),
    );

    // to_json_pretty()
    env.define(
        "to_json_pretty",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "to_json_pretty needs a value".into(),
                ));
            }

            let value = &args[0];
            let json = value_to_json_pretty(value)?;
            Ok(Value::Str(json.into()))
        }),
    );

    // json_valid() - validation of JSON string
    env.define(
        "json_valid",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "json_valid needs a JSON string".into(),
                ));
            }

            let json_str = args[0].as_str();
            let valid = serde_json::from_str::<JsonValue>(&json_str).is_ok();
            Ok(Value::Bool(valid))
        }),
    );
}

fn value_to_json(value: &Value) -> Result<String, RuntimeError> {
    let json_value = crisp_value_to_json(value)?;
    Ok(serde_json::to_string(&json_value)
        .map_err(|e| RuntimeError::ArgumentError(format!("JSON serialization error: {}", e)))?)
}

fn value_to_json_pretty(value: &Value) -> Result<String, RuntimeError> {
    let json_value = crisp_value_to_json(value)?;
    Ok(serde_json::to_string_pretty(&json_value)
        .map_err(|e| RuntimeError::ArgumentError(format!("JSON serialization error: {}", e)))?)
}

fn crisp_value_to_json(value: &Value) -> Result<JsonValue, RuntimeError> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::Int(i) => Ok(JsonValue::Number(Number::from(*i))),
        Value::Float(f) => {
            if let Some(n) = Number::from_f64(*f) {
                Ok(JsonValue::Number(n))
            } else {
                Err(RuntimeError::ArgumentError("Invalid float for JSON".into()))
            }
        }
        Value::Str(s) => Ok(JsonValue::String(s.to_string())),
        Value::Array(arr) => {
            let arr = arr.borrow();
            let elements: Result<Vec<JsonValue>, RuntimeError> =
                arr.iter().map(crisp_value_to_json).collect();
            Ok(JsonValue::Array(elements?))
        }
        Value::Hash(hash) => {
            let hash = hash.borrow();
            let mut map = Map::new();
            for (key, value) in hash.iter() {
                map.insert(key.clone(), crisp_value_to_json(value)?);
            }
            Ok(JsonValue::Object(map))
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

fn json_to_value(json_value: JsonValue) -> Result<Value, RuntimeError> {
    match json_value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(b) => Ok(Value::Bool(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(RuntimeError::ArgumentError("Invalid JSON number".into()))
            }
        }
        JsonValue::String(s) => Ok(Value::Str(s.into())),
        JsonValue::Array(arr) => {
            let elements: Result<Vec<Value>, RuntimeError> =
                arr.into_iter().map(json_to_value).collect();
            Ok(Value::Array(Rc::new(RefCell::new(elements?))))
        }
        JsonValue::Object(map) => {
            let mut hash = IndexMap::new(); 
            for (key, value) in map {
                hash.insert(key, json_to_value(value)?);
            }
            Ok(Value::Hash(Rc::new(RefCell::new(hash))))
        }
    }
}

fn json_to_value_from_str(json_str: &str) -> Result<Value, RuntimeError> {
    let json_value: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| RuntimeError::ArgumentError(format!("Invalid JSON: {}", e)))?;
    json_to_value(json_value)
}
