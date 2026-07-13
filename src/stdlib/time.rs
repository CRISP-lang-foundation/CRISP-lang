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
use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn register(env: &mut Environment) {
    // time() - aktuálny čas v sekundách
    env.define(
        "time",
        Value::NativeFn(|_| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Int(now.as_secs() as i64))
        }),
    );

    // sleep() - zaspí na daný počet sekúnd
    env.define(
        "sleep",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("sleep needs seconds".into()));
            }

            let secs = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as u64;

            std::thread::sleep(std::time::Duration::from_secs(secs));
            Ok(Value::Null)
        }),
    );

    // sleep_ms() - zaspí na daný počet milisekúnd
    env.define(
        "sleep_ms",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "sleep_ms needs milliseconds".into(),
                ));
            }

            let ms = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as u64;

            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(Value::Null)
        }),
    );

    // timestamp() - aktuálny timestamp (milisekúnd)
    env.define(
        "timestamp",
        Value::NativeFn(|_| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Int(now.as_millis() as i64))
        }),
    );

    // datetime() - aktuálny dátum a čas ako reťazec
    env.define(
        "datetime",
        Value::NativeFn(|_| {
            let now = Local::now();
            let date_str = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );

            Ok(Value::Str(date_str.into()))
        }),
    );

    // datetime_utc() - aktuálny UTC čas
    env.define(
        "datetime_utc",
        Value::NativeFn(|_| {
            let now = Utc::now();
            let date_str = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );

            Ok(Value::Str(date_str.into()))
        }),
    );

    // strftime() - formátovanie času
    env.define(
        "strftime",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "strftime needs format and time".into(),
                ));
            }

            let format = args[0].as_str();
            let timestamp = args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as i64;

            let dt = DateTime::from_timestamp(timestamp, 0)
                .ok_or(RuntimeError::ArgumentError("Invalid timestamp".into()))?;

            let local: DateTime<Local> = dt.into();
            let result = local.format(&format).to_string();

            Ok(Value::Str(result.into()))
        }),
    );

    // strptime() - parsovanie času
    env.define(
        "strptime",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "strptime needs string and format".into(),
                ));
            }

            let date_str = args[0].as_str();
            let format = args[1].as_str();

            let dt = NaiveDateTime::parse_from_str(&date_str, &format)
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid date format: {}", e)))?;

            let timestamp = dt.and_utc().timestamp();
            Ok(Value::Int(timestamp))
        }),
    );
}
