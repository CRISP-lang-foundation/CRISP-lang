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
use crate::value::{CmpResult, Value};
use std::f64::consts;

pub fn register(env: &mut Environment) {
    env.define("PI", Value::Float(consts::PI));
    env.define("E", Value::Float(consts::E));
    env.define("TAU", Value::Float(consts::TAU));

    env.define(
        "sqrt",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("sqrt needs a number".into()));
            }
            let num = args[0].as_number().ok_or(RuntimeError::TypeMismatch)?;
            if num < 0.0 {
                return Err(RuntimeError::MathError("sqrt of negative number".into()));
            }
            Ok(Value::Float(num.sqrt()))
        }),
    );

    env.define(
        "pow",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError("pow needs base and exponent".into()));
            }
            let base = args[0].as_number().ok_or(RuntimeError::TypeMismatch)?;
            let exp = args[1].as_number().ok_or(RuntimeError::TypeMismatch)?;

            // Return Int when both args are integers, Float otherwise
            if let (Value::Int(b), Value::Int(e)) = (&args[0], &args[1]) {
                if *e >= 0 && (*e as u64) <= u32::MAX as u64 {
                    return Ok(Value::Int(b.pow(*e as u32)));
                }
            }
            Ok(Value::Float(base.powf(exp)))
        }),
    );

    env.define(
        "abs",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("abs needs a number".into()));
            }
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(RuntimeError::TypeMismatch),
            }
        }),
    );

    env.define(
        "min",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "min needs at least one argument".into(),
                ));
            }
            let mut min = args[0].clone();
            for arg in args.iter().skip(1) {
                if let Ok(CmpResult::Greater) = min.cmp(arg) {
                    min = arg.clone();
                }
            }
            Ok(min)
        }),
    );

    env.define(
        "max",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "max needs at least one argument".into(),
                ));
            }
            let mut max = args[0].clone();
            for arg in args.iter().skip(1) {
                if let Ok(CmpResult::Less) = max.cmp(arg) {
                    max = arg.clone();
                }
            }
            Ok(max)
        }),
    );

    env.define(
        "rand",
        Value::NativeFn(|_| {
            use rand::Rng;
            Ok(Value::Float(rand::random::<f64>()))
        }),
    );
}
