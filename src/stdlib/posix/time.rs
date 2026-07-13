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
    // nanosleep()
    env.define(
        "nanosleep",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "nanosleep needs seconds and nanoseconds".into(),
                ));
            }

            let sec = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as i64;
            let nsec = args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as i64;

            let req = libc::timespec {
                tv_sec: sec,
                tv_nsec: nsec,
            };
            let mut rem = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };

            let result = unsafe { libc::nanosleep(&req, &mut rem) };
            if result == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }

            Ok(Value::Null)
        }),
    );
}
