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
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, Timelike, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn register(env: &mut Environment) {
    // time() - current time in seconds
    env.define(
        "time",
        Value::NativeFn(|_| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Int(now.as_secs() as i64))
        }),
    );

    // sleep() - sleep for a given number of seconds
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

    // sleep_ms() - sleep for a given number of milliseconds
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

    // timestamp() - current timestamp (milliseconds)
    env.define(
        "timestamp",
        Value::NativeFn(|_| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Int(now.as_millis() as i64))
        }),
    );

    // datetime() - current date and time as a string
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

    // datetime_utc() - current UTC time
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

    // strftime() - format a timestamp as a string
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

    // strptime() - parse a string into a timestamp
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

            // Try full datetime first, fall back to date-only
            let dt = NaiveDateTime::parse_from_str(&date_str, &format)
                .or_else(|_| {
                    NaiveDate::parse_from_str(&date_str, &format)
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                })
                .map_err(|e| RuntimeError::ArgumentError(format!("Invalid date format: {}", e)))?;

            let timestamp = dt.and_utc().timestamp();
            Ok(Value::Int(timestamp))
        }),
    );
}
