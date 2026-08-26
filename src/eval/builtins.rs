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

//! Built-in functions for CRISP
//!
//! This module contains basic built-in functions.

use crate::eval::Environment;

pub struct Builtins;

impl Builtins {
    pub fn register(_env: &mut Environment) {
        // Basic functions are already registered in stdlib
        // This is just a placeholder for any additional built-in functions
    }
}
