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
use std::cell::RefCell;
use std::fs::{self};
use std::path::Path;
use std::rc::Rc;

pub fn register(env: &mut Environment) {
    // read_file - read whole file
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
                Ok(content) => Ok(Value::Str(content.into())),
                Err(e) => Err(RuntimeError::IOError(e.to_string())),
            }
        }),
    );

    // write_file - write to file
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

    // file_exists - existence check
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

    // list_dir - list directory contents
    env.define(
        "list_dir",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("list_dir needs a path".into()));
            }
            let path = args[0].as_str();
            let entries: Vec<Value> = fs::read_dir(path)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?
                .filter_map(|entry| entry.ok())
                .map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    Value::Str(name.into())
                })
                .collect();
            Ok(Value::Array(Rc::new(RefCell::new(entries))))
        }),
    );

    // create_dir - create directory (including parent directories)
    env.define(
        "create_dir",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "create_dir needs a path".into(),
                ));
            }
            let path = args[0].as_str();
            match fs::create_dir_all(path) {
                Ok(_) => Ok(Value::Bool(true)),
                Err(e) => Err(RuntimeError::IOError(e.to_string())),
            }
        }),
    );

    // is_dir - check if path is a directory
    env.define(
        "is_dir",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("is_dir needs a path".into()));
            }
            let path = args[0].as_str();
            Ok(Value::Bool(Path::new(&path).is_dir()))
        }),
    );
}
