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
use std::ffi::CString;

pub fn register(env: &mut Environment) {
    // open()
    env.define(
        "open",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "open needs path and flags".into(),
                ));
            }

            let path = args[0].as_str();
            let flags = args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as i32;

            let mode = if args.len() > 2 {
                args[2].as_number().ok_or(RuntimeError::TypeMismatch)? as libc::mode_t
            } else {
                0o666
            };

            let c_path = CString::new(path)
                .map_err(|_| RuntimeError::ArgumentError("invalid path".into()))?;

            let fd = unsafe { libc::open(c_path.as_ptr(), flags, mode) };
            if fd == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }

            Ok(Value::Int(fd as i64))
        }),
    );

    // close()
    env.define(
        "close",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "close needs a file descriptor".into(),
                ));
            }

            let fd = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as i32;

            let result = unsafe { libc::close(fd) };
            if result == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
            Ok(Value::Bool(true))
        }),
    );

    // read()
    env.define(
        "read",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "read needs fd and count".into(),
                ));
            }

            let fd = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as i32;
            let count = args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;

            let mut buffer = vec![0u8; count];
            let result = unsafe { libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, count) };

            if result == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }

            let bytes_read = result as usize;
            buffer.truncate(bytes_read);

            Ok(Value::Str(
                String::from_utf8_lossy(&buffer).to_string().into(),
            ))
        }),
    );

    // write()
    env.define(
        "write",
        Value::NativeFn(|args| {
            if args.len() < 3 {
                return Err(RuntimeError::ArgumentError(
                    "write needs fd, data and count".into(),
                ));
            }

            let fd = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as i32;
            let data = args[1].as_str();
            let count = args[2].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;

            let bytes = data.as_bytes();
            let len = count.min(bytes.len());

            let result = unsafe { libc::write(fd, bytes.as_ptr() as *const libc::c_void, len) };

            if result == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }

            Ok(Value::Int(result as i64))
        }),
    );

    env.define(
        "lseek",
        Value::NativeFn(|args| {
            let fd = args
                .get(0)
                .and_then(|a| a.as_number())
                .ok_or(RuntimeError::TypeMismatch)? as i32;
            let offset = args
                .get(1)
                .and_then(|a| a.as_number())
                .ok_or(RuntimeError::TypeMismatch)? as libc::off_t;
            let whence = args
                .get(2)
                .and_then(|a| a.as_number())
                .ok_or(RuntimeError::TypeMismatch)? as i32;
            let result = unsafe { libc::lseek(fd, offset, whence) };
            if result == -1 {
                Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ))
            } else {
                Ok(Value::Int(result as i64))
            }
        }),
    );
}
