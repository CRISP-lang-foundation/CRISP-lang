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
use std::ffi::CString;
use std::rc::Rc;

use crate::eval::{Environment, RuntimeError};
use crate::value::Value;
use std::process::Command;

pub fn register(env: &mut Environment) {
    // getpid()
    env.define(
        "getpid",
        Value::NativeFn(|_| Ok(Value::Int(unsafe { libc::getpid() } as i64))),
    );

    // getppid()
    env.define(
        "getppid",
        Value::NativeFn(|_| Ok(Value::Int(unsafe { libc::getppid() } as i64))),
    );

    // fork()
    env.define(
        "fork",
        Value::NativeFn(|_| {
            let pid = unsafe { libc::fork() };
            Ok(Value::Int(pid as i64))
        }),
    );

    // waitpid(pid, options) — wait for a child process
    env.define(
        "waitpid",
        Value::NativeFn(|args| {
            let pid = if args.is_empty() {
                -1 // wait for any child
            } else {
                args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as libc::pid_t
            };

            let options = if args.len() > 1 {
                args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as i32
            } else {
                0
            };

            let mut status: i32 = 0;
            let result = unsafe { libc::waitpid(pid, &mut status, options) };

            if result == -1 {
                return Err(RuntimeError::IOError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }

            // Return the exit status
            if libc::WIFEXITED(status) {
                Ok(Value::Int(libc::WEXITSTATUS(status) as i64))
            } else if libc::WIFSIGNALED(status) {
                Ok(Value::Int(-(libc::WTERMSIG(status) as i64)))
            } else {
                Ok(Value::Int(status as i64))
            }
        }),
    );

    // execvp(program, [args...]) — replace current process
    env.define(
        "execvp",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "execvp needs a program name".into(),
                ));
            }

            let program = args[0].as_str();

            // Build argv array
            let mut argv: Vec<CString> = Vec::new();
            argv.push(
                CString::new(program.clone())
                    .map_err(|_| RuntimeError::ArgumentError("invalid program name".into()))?,
            );

            // If second argument is an array, use its elements as args
            if args.len() > 1 {
                match &args[1] {
                    Value::Array(arr) => {
                        for v in arr.borrow().iter() {
                            let s = v.as_str();
                            argv.push(CString::new(s.as_bytes()).map_err(|_| {
                                RuntimeError::ArgumentError("invalid argument string".into())
                            })?);
                        }
                    }
                    _ => {
                        // Single string argument
                        let s = args[1].as_str();
                        argv.push(CString::new(s.as_bytes()).map_err(|_| {
                            RuntimeError::ArgumentError("invalid argument string".into())
                        })?);
                    }
                }
            }

            let c_program = CString::new(program)
                .map_err(|_| RuntimeError::ArgumentError("invalid program".into()))?;

            // Build raw C argv array
            let mut c_argv: Vec<*const libc::c_char> = argv.iter().map(|s| s.as_ptr()).collect();
            c_argv.push(std::ptr::null());

            let result = unsafe { libc::execvp(c_program.as_ptr(), c_argv.as_ptr()) };

            // If we get here, execvp failed
            Err(RuntimeError::IOError(
                std::io::Error::last_os_error().to_string(),
            ))
        }),
    );

    // spawn() — run a command and capture output
    env.define(
        "spawn",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("spawn needs a command".into()));
            }

            let cmd = args[0].as_str();
            let args: Vec<String> = args.iter().skip(1).map(|a| a.as_str()).collect();

            let output = Command::new(cmd)
                .args(&args)
                .output()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let mut hash = std::collections::HashMap::new();
            hash.insert(
                "stdout".into(),
                Value::Str(String::from_utf8_lossy(&output.stdout).to_string().into()),
            );
            hash.insert(
                "stderr".into(),
                Value::Str(String::from_utf8_lossy(&output.stderr).to_string().into()),
            );
            hash.insert(
                "status".into(),
                Value::Int(output.status.code().unwrap_or(-1) as i64),
            );

            Ok(Value::Hash(Rc::new(RefCell::new(hash))))
        }),
    );
}
