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

use crate::eval::{Environment, RuntimeError};
use crate::value::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub fn register(env: &mut Environment) {
    // tcp_connect() - connect to a TCP server
    env.define(
        "tcp_connect",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "tcp_connect needs address".into(),
                ));
            }

            let addr = args[0].as_str();
            let addrs = addr
                .to_socket_addrs()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let stream = TcpStream::connect(addrs.as_slice())
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Store the stream in a hash map and return its ID
            let id = store_stream(stream);
            Ok(Value::Int(id as i64))
        }),
    );

    // tcp_listen() - TCP server
    env.define(
        "tcp_listen",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "tcp_listen needs address".into(),
                ));
            }

            let addr = args[0].as_str();
            let listener =
                TcpListener::bind(addr).map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let id = store_listener(listener);
            Ok(Value::Int(id as i64))
        }),
    );

    // tcp_accept() - accept a connection
    env.define(
        "tcp_accept",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "tcp_accept needs listener ID".into(),
                ));
            }

            let id = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;

            let listener = get_listener(id)
                .ok_or(RuntimeError::ArgumentError("Invalid listener ID".into()))?;

            let (stream, addr) = listener
                .accept()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let stream_id = store_stream(stream);

            let mut result = HashMap::new();
            result.insert("stream".into(), Value::Int(stream_id as i64));
            result.insert("addr".into(), Value::Str(addr.to_string().into()));

            Ok(Value::Hash(Rc::new(RefCell::new(result))))
        }),
    );

    // tcp_read() - read from a socket
    env.define(
        "tcp_read",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "tcp_read needs socket ID and length".into(),
                ));
            }

            let id = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;
            let len = args[1].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;

            let mut stream = get_stream_mut(id)
                .ok_or(RuntimeError::ArgumentError("Invalid socket ID".into()))?;

            let mut buf = vec![0u8; len];
            let n = stream
                .read(&mut buf)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            buf.truncate(n);
            let result = String::from_utf8_lossy(&buf).to_string();

            Ok(Value::Str(result.into()))
        }),
    );

    // tcp_write() - write to a socket
    env.define(
        "tcp_write",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "tcp_write needs socket ID and data".into(),
                ));
            }

            let id = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;
            let data = args[1].as_str();

            let mut stream = get_stream_mut(id)
                .ok_or(RuntimeError::ArgumentError("Invalid socket ID".into()))?;

            let n = stream
                .write(data.as_bytes())
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            stream
                .flush()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Int(n as i64))
        }),
    );

    // tcp_close() - close a socket
    env.define(
        "tcp_close",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError(
                    "tcp_close needs socket ID".into(),
                ));
            }

            let id = args[0].as_number().ok_or(RuntimeError::TypeMismatch)? as usize;

            remove_stream(id);
            Ok(Value::Bool(true))
        }),
    );

    // http_get() - simple HTTP GET
    env.define(
        "http_get",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("http_get needs URL".into()));
            }

            let url = args[0].as_str();

            // Simple implementation - expects http://host:port/path
            let parts: Vec<&str> = url.split("://").collect();
            if parts.len() != 2 || parts[0] != "http" {
                return Err(RuntimeError::ArgumentError("Invalid HTTP URL".into()));
            }

            let rest = parts[1];
            let path_parts: Vec<&str> = rest.splitn(2, '/').collect();
            let host_port = path_parts[0];
            let path = if path_parts.len() > 1 {
                format!("/{}", path_parts[1])
            } else {
                "/".to_string()
            };

            let host_port_parts: Vec<&str> = host_port.split(':').collect();
            let host = host_port_parts[0];
            let port = if host_port_parts.len() > 1 {
                host_port_parts[1].parse::<u16>().unwrap_or(80)
            } else {
                80
            };

            let addr = format!("{}:{}", host, port);
            let mut stream =
                TcpStream::connect(&addr).map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let request = format!(
                "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                path, host
            );

            stream
                .write_all(request.as_bytes())
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;
            stream
                .flush()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(response.into()))
        }),
    );
}

// Global storage for sockets

thread_local! {
    static STREAMS: RefCell<HashMap<usize, TcpStream>> = RefCell::new(HashMap::new());
    static LISTENERS: RefCell<HashMap<usize, TcpListener>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<usize> = RefCell::new(1);
}

fn store_stream(stream: TcpStream) -> usize {
    STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        let id = NEXT_ID.with(|id| {
            let mut id = id.borrow_mut();
            let current = *id;
            *id += 1;
            current
        });
        streams.insert(id, stream);
        id
    })
}

fn store_listener(listener: TcpListener) -> usize {
    LISTENERS.with(|listeners| {
        let mut listeners = listeners.borrow_mut();
        let id = NEXT_ID.with(|id| {
            let mut id = id.borrow_mut();
            let current = *id;
            *id += 1;
            current
        });
        listeners.insert(id, listener);
        id
    })
}

fn get_stream_mut(id: usize) -> Option<TcpStream> {
    STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        streams.remove(&id)
    })
}

fn get_listener(id: usize) -> Option<TcpListener> {
    LISTENERS.with(|listeners| {
        let mut listeners = listeners.borrow_mut();
        listeners.remove(&id)
    })
}

fn remove_stream(id: usize) {
    STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        streams.remove(&id);
    });
}
