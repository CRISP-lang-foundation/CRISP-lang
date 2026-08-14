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
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::rc::Rc;

use crate::eval::{Environment, RuntimeError};
use crate::value::Value;

use indexmap::IndexMap;

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

            let mut result = IndexMap::new();
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

    // http_get() - HTTP GET request using ureq v3
    env.define(
        "http_get",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("http_get needs a URL".into()));
            }
            
            let url = args[0].as_str();

            // Build agent with global timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Make the request
            let response = agent
                .get(&url)
                .call()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
        }),
    );

    // http_get_with_headers() - HTTP GET with custom headers
    env.define(
        "http_get_with_headers",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "http_get_with_headers needs URL and headers hash".into(),
                ));
            }
            
            let url = args[0].as_str();
            let headers = &args[1];

            // Validate headers is a hash
            let headers_hash = match headers {
                Value::Hash(h) => h.borrow(),
                _ => return Err(RuntimeError::TypeMismatch),
            };

            // Build agent with timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Build request with headers using header() method
            let mut request = agent.get(&url);
            
            for (key, value) in headers_hash.iter() {
                let header_value = value.as_str();
                // Convert key to &str to avoid &&String issue
                request = request.header(key.as_str(), &header_value);
            }

            // Make the request
            let response = request
                .call()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
        }),
    );

    // http_post() - HTTP POST request (form data)
    env.define(
        "http_post",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "http_post needs URL and body".into(),
                ));
            }
            
            let url = args[0].as_str();
            let body_content = args[1].as_str();

            // Build agent with timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Make the POST request - use send() instead of send_string()
            let response = agent
                .post(&url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send(body_content)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
        }),
    );

    // http_post_json() - HTTP POST with JSON body
    env.define(
        "http_post_json",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "http_post_json needs URL and JSON body".into(),
                ));
            }
            
            let url = args[0].as_str();
            let json_body = args[1].as_str();

            // Build agent with timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Make the POST request with JSON
            // Note: send_json() requires the "json" feature which is enabled in Cargo.toml
            let response = agent
                .post(&url)
                .header("Content-Type", "application/json")
                .send(json_body)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
        }),
    );

    // http_delete() - HTTP DELETE request
    env.define(
        "http_delete",
        Value::NativeFn(|args| {
            if args.is_empty() {
                return Err(RuntimeError::ArgumentError("http_delete needs a URL".into()));
            }
            
            let url = args[0].as_str();

            // Build agent with timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Make the DELETE request
            let response = agent
                .delete(&url)
                .call()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
        }),
    );

    // http_put() - HTTP PUT request
    env.define(
        "http_put",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "http_put needs URL and body".into(),
                ));
            }
            
            let url = args[0].as_str();
            let body_content = args[1].as_str();

            // Build agent with timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Make the PUT request - use send() instead of send_string()
            let response = agent
                .put(&url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send(body_content)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
        }),
    );

    // http_patch() - HTTP PATCH request
    env.define(
        "http_patch",
        Value::NativeFn(|args| {
            if args.len() < 2 {
                return Err(RuntimeError::ArgumentError(
                    "http_patch needs URL and body".into(),
                ));
            }
            
            let url = args[0].as_str();
            let body_content = args[1].as_str();

            // Build agent with timeout
            let config = ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(30)))
                .build();
                
            let agent = config.new_agent();

            // Make the PATCH request - use send() instead of send_string()
            let response = agent
                .patch(&url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send(body_content)
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            // Read the response body
            let body = response
                .into_body()
                .read_to_string()
                .map_err(|e| RuntimeError::IOError(e.to_string()))?;

            Ok(Value::Str(body.into()))
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
