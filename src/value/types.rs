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

//! Types of values in CRISP

use crate::eval::RuntimeError;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<String>),
    Array(Rc<RefCell<Vec<Value>>>),
    Hash(Rc<RefCell<IndexMap<String, Value>>>),
    Ref(Rc<RefCell<Value>>),
    NativeFn(fn(&[Value]) -> Result<Value, RuntimeError>),
    UserFn {
        name: String,
        params: Vec<String>,
        body: Vec<crate::parser::Stmt>,
        env: Rc<RefCell<crate::eval::Environment>>,
    },
    // --- OOP ---
    Class {
        name: String,
        parent: Option<String>,
        methods: Rc<HashMap<String, Value>>,
        static_methods: Rc<HashMap<String, Value>>,
        env: Rc<RefCell<crate::eval::Environment>>,
    },
    Object {
        class: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: Rc<HashMap<String, Value>>,
    },
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::Hash(h) => !h.borrow().is_empty(),
            _ => true,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Value::Str(s) => s.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let arr = arr.borrow();
                let elements: Vec<String> = arr.iter().map(|v| v.as_str()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Hash(hash) => {
                let hash = hash.borrow();
                let pairs: Vec<String> = hash
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.as_str()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::UserFn { name, params, .. } => {
                format!("<function {}({})>", name, params.join(", "))
            }
            _ => format!("{:?}", self),
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            Value::Str(s) => s.parse::<f64>().ok(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Null => Some(0.0),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
