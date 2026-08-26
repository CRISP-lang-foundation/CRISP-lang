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

use crate::eval::RuntimeError;
use crate::utils::diagnostics;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    variables: HashMap<String, Value>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            parent: None,
            variables: HashMap::new(),
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        if diagnostics::is_debug_mode() {
            diagnostics::log_debug(&format!("ENV: Defining {} in environment", name));
        }
        self.variables.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if diagnostics::is_debug_mode() {
            diagnostics::log_debug(&format!("ENV: Looking up {} in environment", name));
        }

        if let Some(val) = self.variables.get(name) {
            if diagnostics::is_debug_mode() {
                diagnostics::log_debug(&format!("ENV: Found in current environment: {}", name));
            }
            return Some(val.clone());
        }

        if let Some(parent) = &self.parent {
            if diagnostics::is_debug_mode() {
                diagnostics::log_debug("ENV: Looking in parent");
            }
            return parent.borrow().get(name);
        }

        if diagnostics::is_debug_mode() {
            diagnostics::log_debug(&format!("ENV: Not found: {}", name));
        }
        None
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if diagnostics::is_debug_mode() {
            diagnostics::log_debug(&format!("ENV: Setting {} = {:?}", name, value));
        }

        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }

    pub fn get_all(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    /// Return all entries in this environment (not parent)
    pub fn entries(&self) -> Vec<(String, Value)> {
        self.variables
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
