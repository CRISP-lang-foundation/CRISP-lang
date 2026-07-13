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
use sha2::{Digest, Sha256, Sha512};

pub fn register(env: &mut Environment) {
    env.define(
        "md5",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("md5 needs a string".into()));
            }
            let text = args[0].as_str();
            // md5::compute vracia Md5Result ktorý implementuje LowerHex
            let result = md5::compute(text.as_bytes());
            Ok(Value::Str(format!("{:x}", result).into()))
        }),
    );

    env.define(
        "sha1",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("sha1 needs a string".into()));
            }
            let text = args[0].as_str();
            use sha1::Sha1;
            let result = Sha1::digest(text.as_bytes());
            Ok(Value::Str(hex::encode(result).into()))
        }),
    );

    env.define(
        "sha256",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("sha256 needs a string".into()));
            }
            let text = args[0].as_str();
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let result = hasher.finalize();
            Ok(Value::Str(hex::encode(result).into()))
        }),
    );

    env.define(
        "sha512",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("sha512 needs a string".into()));
            }
            let text = args[0].as_str();
            let mut hasher = Sha512::new();
            hasher.update(text.as_bytes());
            let result = hasher.finalize();
            Ok(Value::Str(hex::encode(result).into()))
        }),
    );

    env.define(
        "base64_encode",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "base64_encode needs a string".into(),
                ));
            }
            let text = args[0].as_str();
            use base64::{Engine as _, engine::general_purpose};
            Ok(Value::Str(
                general_purpose::STANDARD.encode(text.as_bytes()).into(),
            ))
        }),
    );

    env.define(
        "base64_decode",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "base64_decode needs a string".into(),
                ));
            }
            let text = args[0].as_str();
            use base64::{Engine as _, engine::general_purpose};
            let decoded = general_purpose::STANDARD
                .decode(text)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid base64: {}", e)))?;
            let result = String::from_utf8(decoded)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid UTF-8: {}", e)))?;
            Ok(Value::Str(result.into()))
        }),
    );

    env.define(
        "random_bytes",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "random_bytes needs length".into(),
                ));
            }
            let len = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;
            let bytes: Vec<u8> = (0..len).map(|_| rand::random()).collect();
            Ok(Value::Str(hex::encode(bytes).into()))
        }),
    );
}
