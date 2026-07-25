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
use std::process::{Command, Stdio};

pub fn register(env: &mut Environment) {
    // system() - run a command (like Perl)
    env.define(
        "system",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("system needs a command".into()));
            }

            let cmd = args[0].as_str();
            let args: Vec<String> = args.iter().skip(1).map(|a| a.as_str()).collect();

            let status = Command::new(cmd)
                .args(&args)
                .status()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Int(status.code().unwrap_or(-1) as i64))
        }),
    );

    // exec() - replace current process (like Perl)
    env.define(
        "exec",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("exec needs a command".into()));
            }

            let cmd = args[0].as_str();
            let args: Vec<String> = args.iter().skip(1).map(|a| a.as_str()).collect();

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let err = Command::new(cmd).args(&args).exec();
                return Err(RuntimeError::IOError(err.to_string()));
            }

            #[cfg(not(unix))]
            {
                // Windows fallback
                let status = Command::new(cmd)
                    .args(&args)
                    .status()
                    .map_err(|e| RuntimeError::IOError(e.to_string()))?;

                std::process::exit(status.code().unwrap_or(1));
            }
        }),
    );

    // qx() / backticks - run and capture output (like Perl)
    env.define(
        "qx",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("qx needs a command".into()));
            }

            let cmd = args[0].as_str();
            let args: Vec<String> = args.iter().skip(1).map(|a| a.as_str()).collect();

            let output = Command::new(cmd)
                .args(&args)
                .output()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(Value::Str(stdout.into()))
        }),
    );

    // pid() - alias for getpid() from POSIX
    #[cfg(unix)]
    env.define(
        "pid",
        Value::NativeFn(|_| Ok(Value::Int(unsafe { libc::getpid() } as i64))),
    );

    #[cfg(not(unix))]
    env.define(
        "pid",
        Value::NativeFn(|_| Ok(Value::Int(std::process::id() as i64))),
    );

    // shell() - run via shell (like Perl)
    env.define(
        "shell",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("shell needs a command".into()));
            }

            let cmd = args[0].as_str();

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .status()
                    .map_err(|e| RuntimeError::IOError(e.to_string()))?;

                Ok(Value::Int(status.code().unwrap_or(-1) as i64))
            }

            #[cfg(not(unix))]
            {
                // Windows fallback
                let status = Command::new("cmd")
                    .arg("/c")
                    .arg(cmd)
                    .status()
                    .map_err(|e| RuntimeError::IOError(e.to_string()))?;

                Ok(Value::Int(status.code().unwrap_or(-1) as i64))
            }
        }),
    );
}
