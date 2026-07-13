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

//! I/O module — file and console operations
//!
//! Provides: print, say, warn, readline, read, input,
//!           along with file-system operations.

use crate::eval::{Environment, RuntimeError};
use crate::value::Value;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::rc::Rc;

pub fn register(env: &mut Environment) {
    // ── Console output ────────────────────────────────────────────────

    env.define(
        "print",
        Value::NativeFn(|args| {
            for arg in args {
                print!("{}", arg.as_str());
            }
            io::stdout().flush()?;
            Ok(Value::Null)
        }),
    );

    env.define(
        "say",
        Value::NativeFn(|args| {
            for arg in args {
                print!("{}", arg.as_str());
            }
            println!();
            io::stdout().flush()?;
            Ok(Value::Null)
        }),
    );

    env.define(
        "warn",
        Value::NativeFn(|args| {
            let mut msg = String::new();
            for arg in args {
                msg.push_str(&arg.as_str());
                msg.push(' ');
            }
            eprintln!("Warning: {}", msg.trim());
            Ok(Value::Null)
        }),
    );

    // ── Console input ────────────────────────────────────────────────

    // readline(prompt?) → String
    // Reads a line from stdin. Optional prompt is printed (no newline),
    // stdout is flushed before blocking on input.
    env.define(
        "readline",
        Value::NativeFn(|args| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt.as_str());
                io::stdout().flush()?;
            }

            let mut line = String::new();
            let bytes = io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| RuntimeError::IOError(format!("readline failed: {}", e)))?;

            if bytes == 0 {
                return Ok(Value::Str(Rc::new(String::new())));
            }

            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            Ok(Value::Str(Rc::new(line)))
        }),
    );

    // read(prompt?) → Int | Float | Str
    // Like readline, but auto-detects the type.
    //   "42"     → Int(42)
    //   "3.14"   → Float(3.14)
    //   "hello"  → Str("hello")
    // Empty input → Str("")
    env.define(
        "read",
        Value::NativeFn(|args| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt.as_str());
                io::stdout().flush()?;
            }

            let mut line = String::new();
            let bytes = io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| RuntimeError::IOError(format!("read failed: {}", e)))?;

            if bytes == 0 {
                return Ok(Value::Str(Rc::new(String::new())));
            }

            let trimmed = line.trim().to_string();

            if trimmed.is_empty() {
                return Ok(Value::Str(Rc::new(String::new())));
            }

            if let Ok(i) = trimmed.parse::<i64>() {
                return Ok(Value::Int(i));
            }

            if let Ok(f) = trimmed.parse::<f64>() {
                return Ok(Value::Float(f));
            }

            // Fallback: raw line without trailing newline
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            Ok(Value::Str(Rc::new(line)))
        }),
    );

    // input(prompt?) → String
    // Python-compatible alias for readline.
    env.define(
        "input",
        Value::NativeFn(|args| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt.as_str());
                io::stdout().flush()?;
            }

            let mut line = String::new();
            io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| RuntimeError::IOError(format!("input failed: {}", e)))?;

            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            Ok(Value::Str(Rc::new(line)))
        }),
    );

    // ── File I/O ──────────────────────────────────────────────────────

    env.define(
        "read_file",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "read_file needs a filename".into(),
                ));
            }

            let filename = args[0].as_str();
            match fs::read_to_string(&filename) {
                Ok(content) => Ok(Value::Str(Rc::new(content))),
                Err(e) => Err(RuntimeError::IOError(e.to_string())),
            }
        }),
    );

    env.define(
        "write_file",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "write_file needs filename and content".into(),
                ));
            }

            let filename = args[0].as_str();
            let content = args[1].as_str();

            match fs::write(&filename, content) {
                Ok(_) => Ok(Value::Bool(true)),
                Err(e) => Err(RuntimeError::IOError(e.to_string())),
            }
        }),
    );

    env.define(
        "file_exists",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "file_exists needs a filename".into(),
                ));
            }

            let filename = args[0].as_str();
            Ok(Value::Bool(Path::new(&filename).exists()))
        }),
    );
}
