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
use std::rc::Rc;

mod environment;
mod files;
mod permissions;
mod process;
mod resources;
mod signals;
mod syslog;
mod terminal;
mod time;
mod users;

pub use environment::*;
pub use files::*;
pub use process::*;
pub use users::*;

use crate::eval::Environment;
use crate::value::Value;

pub fn register(env: &mut Environment) {
    process::register(env);
    users::register(env);
    files::register(env);
    environment::register(env);
    signals::register(env);
    time::register(env);
    terminal::register(env);
    permissions::register(env);
    resources::register(env);
    syslog::register(env);
    register_constants(env);
}

fn register_constants(env: &mut Environment) {
    env.define("SIGINT", Value::Int(libc::SIGINT as i64));
    env.define("SIGTERM", Value::Int(libc::SIGTERM as i64));
    env.define("SIGKILL", Value::Int(libc::SIGKILL as i64));
    env.define("SIGSTOP", Value::Int(libc::SIGSTOP as i64));
    env.define("O_RDONLY", Value::Int(libc::O_RDONLY as i64));
    env.define("O_WRONLY", Value::Int(libc::O_WRONLY as i64));
    env.define("O_RDWR", Value::Int(libc::O_RDWR as i64));
    env.define("O_CREAT", Value::Int(libc::O_CREAT as i64));
    env.define("O_TRUNC", Value::Int(libc::O_TRUNC as i64));
    env.define("O_APPEND", Value::Int(libc::O_APPEND as i64));
}
