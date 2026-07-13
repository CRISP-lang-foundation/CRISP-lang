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

pub fn register(env: &mut Environment) {
    env.define(
        "kill",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "kill needs pid and signal".into(),
                ));
            }
            let pid = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as libc::pid_t;
            let sig = args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as i32;
            let result = unsafe { libc::kill(pid, sig) };
            if result == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
            Ok(Value::Bool(true))
        }),
    );

    env.define(
        "alarm",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("alarm needs seconds".into()));
            }
            let secs = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as u32;
            let remaining = unsafe { libc::alarm(secs) };
            Ok(Value::Int(remaining as i64))
        }),
    );
}
