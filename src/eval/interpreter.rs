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
use crate::parser::{BinaryOp, Expr, MatchPattern, Program, Stmt};
use crate::stdlib;
use crate::utils::diagnostics;
use crate::value::Value;

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
    recursion_depth: usize,
}

const MAX_RECURSION_DEPTH: usize = 1000;

/// Safe debug formatter — avoids infinite recursion from Rc cycles
fn value_fmt(val: &Value) -> String {
    match val {
        Value::UserFn { name, params, .. } => {
            format!("fn {}({})", name, params.join(", "))
        }
        Value::NativeFn(_) => "<native fn>".to_string(),
        Value::Ref(rc) => format!("Ref({})", value_fmt(&rc.borrow())),
        _ => format!("{:?}", val),
    }
}

fn result_fmt(result: &Result<Value, RuntimeError>) -> String {
    match result {
        Ok(v) => value_fmt(v),
        Err(e) => format!("{:?}", e),
    }
}

fn to_usize(val: &Value, max: usize) -> Result<usize, RuntimeError> {
    match val {
        Value::Int(i) => {
            let i = if *i < 0 { 0 } else { *i as usize };
            if i > max {
                Err(RuntimeError::PointerOutOfBounds)
            } else {
                Ok(i)
            }
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        stdlib::register_all(&mut env);
        Interpreter {
            env: Rc::new(RefCell::new(env)),
            recursion_depth: 0,
        }
    }

    pub fn with_env(env: Environment) -> Self {
        Interpreter {
            env: Rc::new(RefCell::new(env)),
            recursion_depth: 0,
        }
    }

    pub fn eval(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut result = Value::Null;
        for stmt in &program.statements {
            match self.eval_statement(stmt) {
                Ok(v) => result = v,
                Err(RuntimeError::BreakSignal) => {
                    return Err(RuntimeError::InvalidOperation(
                        "break outside loop".into(),
                    ));
                }
                Err(RuntimeError::ContinueSignal) => {
                    return Err(RuntimeError::InvalidOperation(
                        "continue outside loop".into(),
                    ));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }

    pub fn eval_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Let { name, expr, .. } => {
                let value = if let Some(e) = expr {
                    self.eval_expression(e)?
                } else {
                    Value::Null
                };
                self.env.borrow_mut().define(name, value);
                Ok(Value::Null)
            }
	    Stmt::Assign { name, expr } => {
                let value = self.eval_expression(expr)?;
                self.env.borrow_mut().set(name, value)?;
                Ok(Value::Null)
            }
            Stmt::Fn { name, params, body } => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!(
                        "DEF: Defining function '{}' with {} parameters",
                        name,
                        params.len()
                    );
                }
                let captured_env =
                    Rc::new(RefCell::new(Environment::with_parent(Rc::clone(&self.env))));
                let func = Value::UserFn {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                    env: captured_env,
                };
                self.env.borrow_mut().define(name, func);
                Ok(Value::Null)
            }
            Stmt::Print(exprs) => {
                for e in exprs {
                    let value = self.eval_expression(e)?;
                    print!("{}", value.as_str());
                }
                use std::io::Write;
                std::io::stdout().flush().ok();
                Ok(Value::Null)
            }
            Stmt::Say(exprs) => {
                for e in exprs {
                    let value = self.eval_expression(e)?;
                    print!("{}", value.as_str());
                }
                println!();
                Ok(Value::Null)
            }

            Stmt::Die(expr) => {
                let value = self.eval_expression(expr)?;
                Err(RuntimeError::UserError(value.as_str()))
            }
            Stmt::Throw(expr) => {
                let value = self.eval_expression(expr)?;
                Err(RuntimeError::UserError(value.as_str()))
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond = self.eval_expression(condition)?;
                if cond.as_bool() {
                    for stmt in then_block {
                        self.eval_statement(stmt)?;
                    }
                } else if let Some(block) = else_block {
                    for stmt in block {
                        self.eval_statement(stmt)?;
                    }
                }
                Ok(Value::Null)
            }
            Stmt::While { condition, body } => {
                while self.eval_expression(condition)?.as_bool() {
                    let mut break_loop = false;
                    for stmt in body {
                        match self.eval_statement(stmt) {
                            Ok(_) => {}
                            Err(RuntimeError::BreakSignal) => {
                                break_loop = true;
                                break;
                            }
                            Err(RuntimeError::ContinueSignal) => {
                                break; // exit inner for, continue outer while
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    if break_loop {
                        break;
                    }
                }
                Ok(Value::Null)
            }
            Stmt::For {
                variable,
                iterable,
                body,
            } => {
                let iter_value = self.eval_expression(iterable)?;
                match iter_value {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        let mut break_loop = false;
                        for elem in arr.iter() {
                            let old_env = Rc::clone(&self.env);
                            let new_env = Environment::with_parent(Rc::clone(&old_env));
                            self.env = Rc::new(RefCell::new(new_env));
                            self.env.borrow_mut().define(variable, elem.clone());

                            let mut continue_outer = false;
                            for stmt in body {
                                match self.eval_statement(stmt) {
                                    Ok(_) => {}
                                    Err(RuntimeError::BreakSignal) => {
                                        break_loop = true;
                                        break;
                                    }
                                    Err(RuntimeError::ContinueSignal) => {
                                        continue_outer = true;
                                        break;
                                    }
                                    Err(e) => {
                                        self.env = old_env;
                                        return Err(e);
                                    }
                                }
                            }
                            self.env = old_env;
                            if break_loop {
                                break;
                            }
                            if continue_outer {
                                continue;
                            }
                        }
                    }
                    Value::Hash(hash) => {
                        let hash = hash.borrow();
                        let mut break_loop = false;
                        for (_key, value) in hash.iter() {
                            let old_env = Rc::clone(&self.env);
                            let new_env = Environment::with_parent(Rc::clone(&old_env));
                            self.env = Rc::new(RefCell::new(new_env));
                            self.env.borrow_mut().define(variable, value.clone());

                            let mut continue_outer = false;
                            for stmt in body {
                                match self.eval_statement(stmt) {
                                    Ok(_) => {}
                                    Err(RuntimeError::BreakSignal) => {
                                        break_loop = true;
                                        break;
                                    }
                                    Err(RuntimeError::ContinueSignal) => {
                                        continue_outer = true;
                                        break;
                                    }
                                    Err(e) => {
                                        self.env = old_env;
                                        return Err(e);
                                    }
                                }
                            }
                            self.env = old_env;
                            if break_loop {
                                break;
                            }
                            if continue_outer {
                                continue;
                            }
                        }
                    }
                    Value::Str(s) => {
                        let mut break_loop = false;
                        for ch in s.chars() {
                            let old_env = Rc::clone(&self.env);
                            let new_env = Environment::with_parent(Rc::clone(&old_env));
                            self.env = Rc::new(RefCell::new(new_env));
                            self.env
                                .borrow_mut()
                                .define(variable, Value::Str(ch.to_string().into()));

                            let mut continue_outer = false;
                            for stmt in body {
                                match self.eval_statement(stmt) {
                                    Ok(_) => {}
                                    Err(RuntimeError::BreakSignal) => {
                                        break_loop = true;
                                        break;
                                    }
                                    Err(RuntimeError::ContinueSignal) => {
                                        continue_outer = true;
                                        break;
                                    }
                                    Err(e) => {
                                        self.env = old_env;
                                        return Err(e);
                                    }
                                }
                            }
                            self.env = old_env;
                            if break_loop {
                                break;
                            }
                            if continue_outer {
                                continue;
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError::TypeMismatch);
                    }
                }
                Ok(Value::Null)
            }
            Stmt::Return(expr) => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("RETURN: Processing return");
                }
                let value = if let Some(e) = expr {
                    self.eval_expression(e)?
                } else {
                    Value::Null
                };
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("RETURN: Returning value: {}", value_fmt(&value));
                }
                Err(RuntimeError::ReturnSignal(value))
            }
            Stmt::Break => Err(RuntimeError::BreakSignal),
            Stmt::Continue => Err(RuntimeError::ContinueSignal),
            Stmt::Expr(expr) => {
                let _result = self.eval_expression(expr)?;
                Ok(Value::Null)
            }
            Stmt::Block(block) => {
                let mut result = Value::Null;
                for stmt in block {
                    result = self.eval_statement(stmt)?;
                }
                Ok(result)
            }
                        Stmt::Use(module) => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("Use module: {}", module);
                }

                match module.as_str() {
                    #[cfg(unix)]
                    "posix" => {
                        let mut temp_env = Environment::new();
                        crate::stdlib::posix::register(&mut temp_env);
                        let mut hash = std::collections::HashMap::new();
                        for (name, func) in temp_env.entries() {
                            hash.insert(name, func);
                        }
                        self.env.borrow_mut().define(
                            "posix",
                            Value::Hash(Rc::new(RefCell::new(hash))),
                        );
                    }
                    _ => {
                        // Unknown module – no-op for now
                    }
                }
                Ok(Value::Null)
            }
            Stmt::Const { name, expr, .. } => {
                let value = self.eval_expression(expr)?;
                self.env.borrow_mut().define(name, value);
                Ok(Value::Null)
            }

            // ─── Try/Catch/Finally ─────────────────────────────────────
            Stmt::TryCatch {
                try_block,
                catch_var,
                catch_block,
                finally_block,
            } => {
                let mut error_val: Option<Value> = None;
                let mut return_signal: Option<Value> = None;
                let mut break_signal = false;
                let mut continue_signal = false;

                for stmt in try_block {
                    match self.eval_statement(stmt) {
                        Ok(_) => {}
                        Err(RuntimeError::ReturnSignal(v)) => {
                            return_signal = Some(v);
                            break;
                        }
                        Err(RuntimeError::BreakSignal) => {
                            break_signal = true;
                            break;
                        }
                        Err(RuntimeError::ContinueSignal) => {
                            continue_signal = true;
                            break;
                        }
                        Err(e) => {
                            error_val = Some(Value::Str(e.to_string().into()));
                            break;
                        }
                    }
                }

                // Only catch real errors, not control-flow signals
                if error_val.is_some()
                    && !break_signal
                    && !continue_signal
                    && return_signal.is_none()
                {
                    if let Some(catch_stmts) = catch_block {
                        let saved_env = if catch_var.is_some() {
                            let old = Rc::clone(&self.env);
                            let new_env = Environment::with_parent(Rc::clone(&old));
                            self.env = Rc::new(RefCell::new(new_env));
                            Some(old)
                        } else {
                            None
                        };

                        if let Some(var) = catch_var {
                            self.env
                                .borrow_mut()
                                .define(var, error_val.as_ref().unwrap().clone());
                        }

                        for stmt in catch_stmts {
                            match self.eval_statement(stmt) {
                                Ok(_) => {}
                                Err(RuntimeError::ReturnSignal(v)) => {
                                    return_signal = Some(v);
                                    break;
                                }
                                Err(RuntimeError::BreakSignal) => {
                                    break_signal = true;
                                    break;
                                }
                                Err(RuntimeError::ContinueSignal) => {
                                    continue_signal = true;
                                    break;
                                }
                                Err(e) => {
                                    if let Some(old) = saved_env {
                                        self.env = old;
                                    }
                                    // Execute finally before re-raising
                                    if let Some(finally) = finally_block {
                                        for s in finally {
                                            let _ = self.eval_statement(s);
                                        }
                                    }
                                    return Err(e);
                                }
                            }
                        }

                        if let Some(old) = saved_env {
                            self.env = old;
                        }
                    }
                }

                // Finally always executes
                if let Some(finally_stmts) = finally_block {
                    for s in finally_stmts {
                        self.eval_statement(s)?;
                    }
                }

                // Re-emit control-flow signals after finally
                if break_signal {
                    return Err(RuntimeError::BreakSignal);
                }
                if continue_signal {
                    return Err(RuntimeError::ContinueSignal);
                }
                if let Some(v) = return_signal {
                    return Err(RuntimeError::ReturnSignal(v));
                }

                Ok(Value::Null)
            }
            Stmt::Match { value, arms } => {
                let val = self.eval_expression(value)?;

                for arm in arms {
                    let pattern_matches = match &arm.pattern {
                        MatchPattern::Wildcard => true,
                        MatchPattern::Var(_) => true,
                        MatchPattern::Int(i) => {
                            if let Value::Int(v) = &val { v == i } else { false }
                        }
                        MatchPattern::Str(s) => val.as_str() == *s,
                        MatchPattern::Bool(b) => {
                            if let Value::Bool(v) = &val { v == b } else { false }
                        }
                    };

                    if !pattern_matches {
                        continue;
                    }

                    if let Some(guard_expr) = &arm.guard {
                        let old_env = Rc::clone(&self.env);
                        let new_env = Environment::with_parent(Rc::clone(&old_env));
                        self.env = Rc::new(RefCell::new(new_env));
                        if let MatchPattern::Var(ref name) = arm.pattern {
                            self.env.borrow_mut().define(name, val.clone());
                        }
                        let guard_ok = self.eval_expression(guard_expr)?.as_bool();
                        self.env = old_env;
                        if !guard_ok {
                            continue;
                        }
                    }

                    // Bind pattern variable and execute EVERY statement in the arm body
                    let old_env = Rc::clone(&self.env);
                    let new_env = Environment::with_parent(Rc::clone(&old_env));
                    self.env = Rc::new(RefCell::new(new_env));
                    if let MatchPattern::Var(ref name) = arm.pattern {
                        self.env.borrow_mut().define(name, val.clone());
                    }

                    let mut result = Value::Null;
                    for stmt in &arm.body {
                        match self.eval_statement(stmt) {
                            Ok(v) => {
                                result = v;
                                // keep going — don't return yet
                            }
                            Err(RuntimeError::ReturnSignal(v)) => {
                                self.env = old_env;
                                return Ok(v);
                            }
                            Err(RuntimeError::BreakSignal) => {
                                self.env = old_env;
                                return Err(RuntimeError::BreakSignal);
                            }
                            Err(RuntimeError::ContinueSignal) => {
                                self.env = old_env;
                                return Err(RuntimeError::ContinueSignal);
                            }
                            Err(e) => {
                                self.env = old_env;
                                return Err(e);
                            }
                        }
                    }
                    self.env = old_env;
                    return Ok(result);
                }

                Ok(Value::Null)
            }
        }
    }

    pub fn eval_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Int(i) => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("INT: {}", i);
                }
                Ok(Value::Int(*i))
            }
            Expr::Float(f) => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("FLOAT: {}", f);
                }
                Ok(Value::Float(*f))
            }
            Expr::Str(s) => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("STR: {}", s);
                }
                Ok(Value::Str(s.clone().into()))
            }
            Expr::Bool(b) => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("BOOL: {}", b);
                }
                Ok(Value::Bool(*b))
            }
            Expr::Null => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("NULL");
                }
                Ok(Value::Null)
            }
            Expr::Var { name, .. } => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("VAR: Looking up '{}'", name);
                }
                let result = self
                    .env
                    .borrow()
                    .get(name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()));
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    match &result {
                        Ok(val) => eprintln!("VAR: '{}' -> {}", name, value_fmt(val)),
                        Err(e) => eprintln!("VAR: '{}' not found! {:?}", name, e),
                    }
                }
                result
            }
            Expr::Binary { left, op, right } => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("BINARY: {:?} {:?} {:?}", left, op, right);
                }
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;
                let result = self.eval_binary_op(left_val, op, right_val);
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("BINARY: Result = {}", result_fmt(&result));
                }
                result
            }
            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for elem in elements {
                    arr.push(self.eval_expression(elem)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(arr))))
            }
            Expr::Hash(pairs) => {
                let mut hash = std::collections::HashMap::new();
                for (key, value) in pairs {
                    let key_str = self.eval_expression(key)?.as_str();
                    let value = self.eval_expression(value)?;
                    hash.insert(key_str, value);
                }
                Ok(Value::Hash(Rc::new(RefCell::new(hash))))
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond = self.eval_expression(condition)?;
                if cond.as_bool() {
                    self.eval_expression(then_expr)
                } else {
                    self.eval_expression(else_expr)
                }
            }
            Expr::Lambda { params, body } => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("LAMBDA: Creating lambda with {} parameters", params.len());
                }
                let captured_env =
                    Rc::new(RefCell::new(Environment::with_parent(Rc::clone(&self.env))));
                Ok(Value::UserFn {
                    name: "lambda".to_string(),
                    params: params.clone(),
                    body: body.clone(),
                    env: captured_env,
                })
            }
            Expr::FieldAccess { object, field } => {
                let obj = self.eval_expression(object)?;
                match obj {
                    Value::Hash(hash) => {
                        let hash = hash.borrow();
                        if let Some(value) = hash.get(field.as_str()) {
                            Ok(value.clone())
                        } else {
                            Err(RuntimeError::UndefinedVariable(format!(
                                "Field '{}' not found",
                                field
                            )))
                        }
                    }
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Index { collection, index } => {
                let coll = self.eval_expression(collection)?;
                let idx = self.eval_expression(index)?;
                match &coll {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        if let Value::Int(i) = idx {
                            if i < 0 || i as usize >= arr.len() {
                                return Err(RuntimeError::PointerOutOfBounds);
                            }
                            Ok(arr[i as usize].clone())
                        } else {
                            Err(RuntimeError::TypeMismatch)
                        }
                    }
                    Value::Hash(hash) => {
                        let hash = hash.borrow();
                        let key = idx.as_str();
                        hash.get(&key).cloned().ok_or_else(|| {
                            RuntimeError::UndefinedVariable(format!("Key '{}' not found", key))
                        })
                    }
                    Value::Str(s) => {
                        if let Value::Int(i) = idx {
                            let ch = s.chars().nth(i as usize).unwrap_or('\0');
                            Ok(Value::Str(ch.to_string().into()))
                        } else {
                            Err(RuntimeError::TypeMismatch)
                        }
                    }
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Slice {
                collection,
                start,
                end,
            } => {
                let coll = self.eval_expression(collection)?;
                let start_idx = start
                    .as_ref()
                    .map(|s| self.eval_expression(s))
                    .transpose()?;
                let end_idx = end.as_ref().map(|e| self.eval_expression(e)).transpose()?;

                let len = match &coll {
                    Value::Array(arr) => arr.borrow().len(),
                    Value::Str(s) => s.len(),
                    _ => return Err(RuntimeError::TypeMismatch),
                };

                let s = to_usize(&start_idx.unwrap_or(Value::Int(0)), len)?;
                let e = to_usize(&end_idx.unwrap_or(Value::Int(len as i64)), len)?;

                if s > e {
                    return Err(RuntimeError::ArgumentError(
                        "Invalid slice range: start > end".into(),
                    ));
                }

                match &coll {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        let slice: Vec<Value> = arr[s..e].to_vec();
                        Ok(Value::Array(Rc::new(RefCell::new(slice))))
                    }
                    Value::Str(string) => {
                        let slice = &string[s..e];
                        Ok(Value::Str(slice.to_string().into()))
                    }
                    _ => unreachable!(),
                }
            }
            Expr::HashAccess { hash, key } => {
                let h_val = self.eval_expression(hash)?;
                let k = self.eval_expression(key)?.as_str();
                match h_val {
                    Value::Hash(h) => {
                        let h = h.borrow();
                        h.get(&k).cloned().ok_or_else(|| {
                            RuntimeError::UndefinedVariable(format!("Key '{}' not found", k))
                        })
                    }
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Method {
                object,
                method,
                args,
            } => {
                let obj = self.eval_expression(object)?;

                let mut call_args: Vec<Value> = vec![obj.clone()];
                for a in args {
                    call_args.push(self.eval_expression(a)?);
                }

                match method.as_str() {
                    "len" => match &call_args[0] {
                        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
                        Value::Array(a) => Ok(Value::Int(a.borrow().len() as i64)),
                        Value::Hash(h) => Ok(Value::Int(h.borrow().len() as i64)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    "split" => {
                        let delim = if call_args.len() > 1 {
                            call_args[1].as_str()
                        } else {
                            " ".to_string()
                        };
                        let s = call_args[0].as_str();
                        let parts: Vec<Value> = s
                            .split(&delim)
                            .map(|p| Value::Str(p.to_string().into()))
                            .collect();
                        Ok(Value::Array(Rc::new(RefCell::new(parts))))
                    }
                    "join" => {
                        if call_args.len() < 2 {
                            return Err(RuntimeError::ArgumentError(
                                "join() requires a separator argument".into(),
                            ));
                        }
                        let sep = &call_args[1].as_str();
                        let arr = match &call_args[0] {
                            Value::Array(a) => a.borrow(),
                            _ => return Err(RuntimeError::TypeMismatch),
                        };
                        let joined: String =
                            arr.iter().map(|v| v.as_str()).collect::<Vec<_>>().join(sep);
                        Ok(Value::Str(joined.into()))
                    }
                    "push" => match &call_args[0] {
                        Value::Array(a) => {
                            for val in &call_args[1..] {
                                a.borrow_mut().push(val.clone());
                            }
                            Ok(Value::Int(a.borrow().len() as i64))
                        }
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    "pop" => match &call_args[0] {
                        Value::Array(a) => Ok(a.borrow_mut().pop().unwrap_or(Value::Null)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    "shift" => match &call_args[0] {
                        Value::Array(a) => {
                            if a.borrow().is_empty() {
                                Ok(Value::Null)
                            } else {
                                Ok(a.borrow_mut().remove(0))
                            }
                        }
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    "unshift" => match &call_args[0] {
                        Value::Array(a) => {
                            for val in call_args[1..].iter().rev() {
                                a.borrow_mut().insert(0, val.clone());
                            }
                            Ok(Value::Int(a.borrow().len() as i64))
                        }
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    "map" => {
                        if call_args.len() < 2 {
                            return Err(RuntimeError::ArgumentError(
                                "map() requires a callback function".into(),
                            ));
                        }
                        let callback = &call_args[1];
                        match &call_args[0] {
                            Value::Array(arr) => {
                                let arr = arr.borrow();
                                let mut result = Vec::new();
                                for elem in arr.iter() {
                                    let mapped = self.call_value(callback, &[elem.clone()])?;
                                    result.push(mapped);
                                }
                                Ok(Value::Array(Rc::new(RefCell::new(result))))
                            }
                            Value::Hash(hash) => {
                                let hash = hash.borrow();
                                let mut result = Vec::new();
                                for (k, v) in hash.iter() {
                                    let mapped = self.call_value(
                                        callback,
                                        &[Value::Str(k.clone().into()), v.clone()],
                                    )?;
                                    result.push(mapped);
                                }
                                Ok(Value::Array(Rc::new(RefCell::new(result))))
                            }
                            _ => Err(RuntimeError::TypeMismatch),
                        }
                    }
                    "filter" | "grep" => {
                        if call_args.len() < 2 {
                            return Err(RuntimeError::ArgumentError(
                                "filter() requires a callback function".into(),
                            ));
                        }
                        let callback = &call_args[1];
                        match &call_args[0] {
                            Value::Array(arr) => {
                                let arr = arr.borrow();
                                let mut result = Vec::new();
                                for elem in arr.iter() {
                                    let keep = self.call_value(callback, &[elem.clone()])?;
                                    if keep.as_bool() {
                                        result.push(elem.clone());
                                    }
                                }
                                Ok(Value::Array(Rc::new(RefCell::new(result))))
                            }
                            Value::Hash(hash) => {
                                let hash = hash.borrow();
                                let mut result = std::collections::HashMap::new();
                                for (k, v) in hash.iter() {
                                    let keep = self.call_value(
                                        callback,
                                        &[Value::Str(k.clone().into()), v.clone()],
                                    )?;
                                    if keep.as_bool() {
                                        result.insert(k.clone(), v.clone());
                                    }
                                }
                                Ok(Value::Hash(Rc::new(RefCell::new(result))))
                            }
                            _ => Err(RuntimeError::TypeMismatch),
                        }
                    }
                    "keys" => match &call_args[0] {
                        Value::Hash(h) => {
                            let keys: Vec<Value> = h
                                .borrow()
                                .keys()
                                .map(|k| Value::Str(k.clone().into()))
                                .collect();
                            Ok(Value::Array(Rc::new(RefCell::new(keys))))
                        }
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    "values" => match &call_args[0] {
                        Value::Hash(h) => {
                            let values: Vec<Value> = h.borrow().values().cloned().collect();
                            Ok(Value::Array(Rc::new(RefCell::new(values))))
                        }
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    _ => match &call_args[0] {
                        Value::Hash(h) => {
                            let h = h.borrow();
                            if let Some(Value::NativeFn(f)) = h.get(method.as_str()) {
                                f(&call_args)
                            } else if let Some(user_fn) = h.get(method.as_str()) {
                                if let Value::UserFn {
                                    params, body, env, ..
                                } = user_fn
                                {
                                    let new_env = Environment::with_parent(env.clone());
                                    let mut interpreter = Interpreter::with_env(new_env);
                                    for (i, param) in params.iter().enumerate() {
                                        if i < call_args.len() {
                                            interpreter
                                                .env
                                                .borrow_mut()
                                                .define(param, call_args[i].clone());
                                        }
                                    }
                                    let mut result = Value::Null;
                                    for stmt in body {
                                        match interpreter.eval_statement(stmt) {
                                            Ok(val) => result = val,
                                            Err(RuntimeError::ReturnSignal(v)) => {
                                                result = v;
                                                break;
                                            }
                                            Err(RuntimeError::BreakSignal) => {
                                                return Err(RuntimeError::InvalidOperation(
                                                    "break outside loop".into(),
                                                ));
                                            }
                                            Err(RuntimeError::ContinueSignal) => {
                                                return Err(RuntimeError::InvalidOperation(
                                                    "continue outside loop".into(),
                                                ));
                                            }
                                            Err(e) => return Err(e),
                                        }
                                    }
                                    Ok(result)
                                } else {
                                    Err(RuntimeError::UndefinedVariable(format!(
                                        "'{}' is not callable",
                                        method
                                    )))
                                }
                            } else {
                                Err(RuntimeError::UndefinedVariable(format!(
                                    "Method '{}' not found",
                                    method
                                )))
                            }
                        }
                        _ => Err(RuntimeError::UndefinedVariable(format!(
                            "Method '{}' not supported on this type",
                            method
                        ))),
                    },
                }
            }
            Expr::Ref(inner) => {
                let val = self.eval_expression(inner)?;
                Ok(Value::Ref(Rc::new(RefCell::new(val))))
            }
            Expr::Deref(inner) => {
                let ptr = self.eval_expression(inner)?;
                match ptr {
                    Value::Ref(rc) => Ok(rc.borrow().clone()),
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                let s = self.eval_expression(start)?;
                let e = self.eval_expression(end)?;
                match (&s, &e) {
                    (Value::Int(a), Value::Int(b)) => {
                        let range: Vec<Value> = if *inclusive {
                            (*a..=*b).map(Value::Int).collect()
                        } else {
                            (*a..*b).map(Value::Int).collect()
                        };
                        Ok(Value::Array(Rc::new(RefCell::new(range))))
                    }
                    (Value::Str(a), Value::Str(b)) if a.len() == 1 && b.len() == 1 => {
                        let start_char = a.chars().next().unwrap() as u32;
                        let end_char = b.chars().next().unwrap() as u32;
                        let range: Vec<Value> = if *inclusive {
                            (start_char..=end_char)
                                .map(|c| Value::Str(char::from_u32(c).unwrap().to_string().into()))
                                .collect()
                        } else {
                            (start_char..end_char)
                                .map(|c| Value::Str(char::from_u32(c).unwrap().to_string().into()))
                                .collect()
                        };
                        Ok(Value::Array(Rc::new(RefCell::new(range))))
                    }
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Concat { left, right } => {
                let l = self.eval_expression(left)?;
                let r = self.eval_expression(right)?;
                Ok(Value::Str(format!("{}{}", l.as_str(), r.as_str()).into()))
            }
            Expr::Repeat { expr, count } => {
                let v = self.eval_expression(expr)?;
                let cnt = self.eval_expression(count)?;
                match cnt {
                    Value::Int(n) => {
                        if n < 0 {
                            return Err(RuntimeError::ArgumentError(
                                "Repeat count must be non-negative".into(),
                            ));
                        }
                        Ok(Value::Str(v.as_str().repeat(n as usize).into()))
                    }
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Spaceship { left, right } => {
                let l = self.eval_expression(left)?;
                let r = self.eval_expression(right)?;
                if let (Some(ln), Some(rn)) = (l.as_number(), r.as_number()) {
                    if ln < rn {
                        Ok(Value::Int(-1))
                    } else if ln > rn {
                        Ok(Value::Int(1))
                    } else {
                        Ok(Value::Int(0))
                    }
                } else {
                    let ls = l.as_str();
                    let rs = r.as_str();
                    match ls.cmp(&rs) {
                        std::cmp::Ordering::Less => Ok(Value::Int(-1)),
                        std::cmp::Ordering::Equal => Ok(Value::Int(0)),
                        std::cmp::Ordering::Greater => Ok(Value::Int(1)),
                    }
                }
            }
            Expr::Where { condition, body } => {
                if self.eval_expression(condition)?.as_bool() {
                    self.eval_expression(body)
                } else {
                    Ok(Value::Null)
                }
            }
            Expr::Substitute {
                text,
                pattern,
                replacement,
                global,
            } => {
                let t = self.eval_expression(text)?.as_str();
                let p = self.eval_expression(pattern)?.as_str();
                let r = self.eval_expression(replacement)?.as_str();

                let re = regex::Regex::new(&p)
                    .map_err(|e| RuntimeError::InvalidOperation(format!("Invalid regex: {}", e)))?;

                let result = if *global {
                    re.replace_all(&t, r.as_str()).to_string()
                } else {
                    re.replace(&t, r.as_str()).to_string()
                };

                Ok(Value::Str(result.into()))
            }
            Expr::Match {
                left,
                right,
                global,
            } => {
                let text = self.eval_expression(left)?.as_str();
                let pattern = self.eval_expression(right)?.as_str();
                let re = regex::Regex::new(&pattern)
                    .map_err(|e| RuntimeError::InvalidOperation(format!("Invalid regex: {}", e)))?;

                if *global {
                    let matches: Vec<Value> = re
                        .find_iter(&text)
                        .map(|m| Value::Str(m.as_str().to_string().into()))
                        .collect();
                    Ok(Value::Array(Rc::new(RefCell::new(matches))))
                } else {
                    Ok(Value::Bool(re.is_match(&text)))
                }
            }
            Expr::Call { func, args, .. } => {
                if self.recursion_depth > MAX_RECURSION_DEPTH {
                    return Err(RuntimeError::RecursionLimit(format!(
                        "Maximum recursion depth of {} exceeded",
                        MAX_RECURSION_DEPTH
                    )));
                }

                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("CALL: Calling function");
                    if let Expr::Var { name, .. } = func.as_ref() {
                        eprintln!("CALL: Function name: '{}'", name);
                    }
                    eprintln!("CALL: Number of arguments: {}", args.len());
                }

                let func_val = self.eval_expression(func)?;

                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    match &func_val {
                        Value::NativeFn(_) => eprintln!("CALL: Native function"),
                        Value::UserFn { name, params, .. } => {
                            eprintln!(
                                "CALL: User function '{}' with parameters: {:?}",
                                name, params
                            );
                        }
                        other => eprintln!("CALL: Not a function! {}", value_fmt(other)),
                    }
                }

                let mut eval_args = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                        eprintln!("CALL: Evaluating argument {}: {:?}", i, arg);
                    }
                    let arg_val = self.eval_expression(arg)?;
                    if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                        eprintln!("CALL: Argument {} = {}", i, value_fmt(&arg_val));
                    }
                    eval_args.push(arg_val);
                }

                match func_val {
                    Value::NativeFn(f) => {
                        let result = f(&eval_args);
                        if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                            eprintln!("CALL: Native function returned {}", result_fmt(&result));
                        }
                        result
                    }
                    Value::UserFn {
                        name,
                        params,
                        body,
                        env,
                    } => {
                        if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                            eprintln!("CALL: Executing function '{}'", name);
                            eprintln!("CALL: Recursion depth before: {}", self.recursion_depth);
                        }

                        self.recursion_depth += 1;

                        let new_env = Environment::with_parent(env.clone());
                        let mut interpreter = Interpreter::with_env(new_env);

                        for (i, param) in params.iter().enumerate() {
                            if i < eval_args.len() {
                                if diagnostics::get_debug_level()
                                    == diagnostics::DebugLevel::Verbose
                                {
                                    eprintln!(
                                        "CALL: Assigning {} = {}",
                                        param,
                                        value_fmt(&eval_args[i])
                                    );
                                }
                                interpreter
                                    .env
                                    .borrow_mut()
                                    .define(param, eval_args[i].clone());
                            }
                        }

                        if !name.is_empty() && name != "lambda" {
                            let self_ref = Value::UserFn {
                                name: name.clone(),
                                params: params.clone(),
                                body: body.clone(),
                                env: env.clone(),
                            };
                            interpreter.env.borrow_mut().define(&name, self_ref);
                            if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                                eprintln!("CALL: Added self-reference '{}'", name);
                            }
                        }

                        let mut result = Value::Null;
                        for stmt in body {
                            match interpreter.eval_statement(&stmt) {
                                Ok(val) => {
                                    if diagnostics::get_debug_level()
                                        == diagnostics::DebugLevel::Verbose
                                    {
                                        eprintln!("CALL: Statement returned: {}", value_fmt(&val));
                                    }
                                    result = val;
                                }
                                Err(RuntimeError::ReturnSignal(val)) => {
                                    result = val;
                                    if diagnostics::get_debug_level()
                                        == diagnostics::DebugLevel::Verbose
                                    {
                                        eprintln!(
                                            "CALL: Found RETURN with value: {}",
                                            value_fmt(&result)
                                        );
                                    }
                                    break;
                                }
                                Err(RuntimeError::BreakSignal) => {
                                    self.recursion_depth -= 1;
                                    return Err(RuntimeError::InvalidOperation(
                                        "break outside loop".into(),
                                    ));
                                }
                                Err(RuntimeError::ContinueSignal) => {
                                    self.recursion_depth -= 1;
                                    return Err(RuntimeError::InvalidOperation(
                                        "continue outside loop".into(),
                                    ));
                                }
                                Err(e) => {
                                    self.recursion_depth -= 1;
                                    if diagnostics::get_debug_level()
                                        == diagnostics::DebugLevel::Verbose
                                    {
                                        eprintln!("CALL: Error: {:?}", e);
                                    }
                                    return Err(e);
                                }
                            }
                        }

                        self.recursion_depth -= 1;
                        if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                            eprintln!("CALL: Function '{}' RETURNS {}", name, value_fmt(&result));
                            eprintln!("CALL: Recursion depth after: {}", self.recursion_depth);
                        }
                        Ok(result)
                    }
                    _ => Err(RuntimeError::TypeMismatch),
                }
            }
            Expr::Unary { op, expr } => {
                let val = self.eval_expression(expr)?;
                match op {
                    crate::parser::UnaryOp::Neg => {
                        if let Some(n) = val.as_number() {
                            Ok(Value::Float(-n))
                        } else {
                            Err(RuntimeError::TypeMismatch)
                        }
                    }
                    crate::parser::UnaryOp::Not => Ok(Value::Bool(!val.as_bool())),
                    crate::parser::UnaryOp::Ref => Ok(Value::Ref(Rc::new(RefCell::new(val)))),
                    crate::parser::UnaryOp::Deref => match val {
                        Value::Ref(rc) => Ok(rc.borrow().clone()),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                    crate::parser::UnaryOp::BitNot => match val {
                        Value::Int(n) => Ok(Value::Int(!n)),
                        _ => Err(RuntimeError::TypeMismatch),
                    },
                }
            }
            _ => {
                if diagnostics::get_debug_level() == diagnostics::DebugLevel::Verbose {
                    eprintln!("UNIMPLEMENTED EXPRESSION: {:?}", expr);
                }
                Ok(Value::Null)
            }
        }
    }

    fn eval_binary_op(
        &self,
        left: Value,
        op: &BinaryOp,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        use BinaryOp::*;
        match (left.clone(), right.clone()) {
            (Value::Int(l), Value::Int(r)) => match op {
                Add => Ok(Value::Int(l + r)),
                Sub => Ok(Value::Int(l - r)),
                Mul => Ok(Value::Int(l * r)),
                Div => {
                    if r == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Int(l / r))
                    }
                }
                Mod => Ok(Value::Int(l % r)),
                Pow => Ok(Value::Int(l.pow(r as u32))),
                Eq => Ok(Value::Bool(l == r)),
                Ne => Ok(Value::Bool(l != r)),
                Lt => Ok(Value::Bool(l < r)),
                Gt => Ok(Value::Bool(l > r)),
                Le => Ok(Value::Bool(l <= r)),
                Ge => Ok(Value::Bool(l >= r)),
                Cmp => Ok(Value::Int(l.cmp(&r) as i64)),
                BitOr => Ok(Value::Int(l | r)),
                BitXor => Ok(Value::Int(l ^ r)),
                BitAnd => Ok(Value::Int(l & r)),
                ShiftLeft => {
                    if r < 0 {
                        return Err(RuntimeError::ArgumentError(
                            "Negative shift count".into(),
                        ));
                    }
                    Ok(Value::Int(l << r))
                }
                ShiftRight => {
                    if r < 0 {
                        return Err(RuntimeError::ArgumentError(
                            "Negative shift count".into(),
                        ));
                    }
                    Ok(Value::Int(l >> r))
                }
                _ => Err(RuntimeError::InvalidOperation(format!("{:?}", op))),
            },
            (Value::Str(l), Value::Str(r)) => match op {
                Add => Ok(Value::Str(format!("{}{}", l, r).into())),
                Concat => Ok(Value::Str(format!("{}{}", l, r).into())),
                Eq => Ok(Value::Bool(l == r)),
                Ne => Ok(Value::Bool(l != r)),
                _ => Err(RuntimeError::InvalidOperation(format!("{:?}", op))),
            },
            (Value::Float(l), Value::Float(r)) => match op {
                Add => Ok(Value::Float(l + r)),
                Sub => Ok(Value::Float(l - r)),
                Mul => Ok(Value::Float(l * r)),
                Div => {
                    if r == 0.0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Float(l / r))
                    }
                }
                Pow => Ok(Value::Float(l.powf(r))),
                Eq => Ok(Value::Bool((l - r).abs() < f64::EPSILON)),
                Ne => Ok(Value::Bool((l - r).abs() >= f64::EPSILON)),
                Lt => Ok(Value::Bool(l < r)),
                Gt => Ok(Value::Bool(l > r)),
                Le => Ok(Value::Bool(l <= r)),
                Ge => Ok(Value::Bool(l >= r)),
                _ => Err(RuntimeError::InvalidOperation(format!("{:?}", op))),
            },
            (Value::Bool(l), Value::Bool(r)) => match op {
                Eq => Ok(Value::Bool(l == r)),
                Ne => Ok(Value::Bool(l != r)),
                _ => Err(RuntimeError::InvalidOperation(format!("{:?}", op))),
            },
            _ => {
                let l_num = left.as_number();
                let r_num = right.as_number();
                if let (Some(l), Some(r)) = (l_num, r_num) {
                    return self.eval_binary_op(Value::Float(l), op, Value::Float(r));
                }

                if matches!(op, Add) {
                    let l_str = left.as_str();
                    let r_str = right.as_str();
                    return Ok(Value::Str(format!("{}{}", l_str, r_str).into()));
                }

                if matches!(op, Eq | Ne) {
                    let l_str = left.as_str();
                    let r_str = right.as_str();
                    let eq = l_str == r_str;
                    return Ok(match op {
                        Eq => Value::Bool(eq),
                        Ne => Value::Bool(!eq),
                        _ => unreachable!(),
                    });
                }

                if matches!(op, Concat) {
                    return Ok(Value::Str(format!("{}{}", left.as_str(), right.as_str()).into()));
                }

		// Null equality
                if matches!(left, Value::Null) || matches!(right, Value::Null) {
                    match op {
                        Eq => return Ok(Value::Bool(matches!(left, Value::Null) && matches!(right, Value::Null))),
                        Ne => return Ok(Value::Bool(!(matches!(left, Value::Null) && matches!(right, Value::Null)))),
                        _ => return Err(RuntimeError::TypeMismatch),
                    }
                }
	
                if matches!(op, Repeat) {
                    if let Value::Int(n) = right {
                        if n >= 0 {
                            return Ok(Value::Str(left.as_str().repeat(n as usize).into()));
                        }
                    }
                    return Err(RuntimeError::TypeMismatch);
                }

                Err(RuntimeError::TypeMismatch)
            }
        }
    }

    fn call_value(&mut self, func: &Value, args: &[Value]) -> Result<Value, RuntimeError> {
        match func {
            Value::NativeFn(f) => f(args),
            Value::UserFn {
                params, body, env, ..
            } => {
                let saved_recursion = self.recursion_depth;
                self.recursion_depth += 1;

                let new_env = Environment::with_parent(env.clone());
                let mut interpreter = Interpreter::with_env(new_env);

                for (i, param) in params.iter().enumerate() {
                    if i < args.len() {
                        interpreter.env.borrow_mut().define(param, args[i].clone());
                    } else {
                        interpreter.env.borrow_mut().define(param, Value::Null);
                    }
                }

                let mut result = Value::Null;
                for stmt in body {
                    match interpreter.eval_statement(stmt) {
                        Ok(val) => result = val,
                        Err(RuntimeError::ReturnSignal(v)) => {
                            result = v;
                            break;
                        }
                        Err(RuntimeError::BreakSignal) => {
                            self.recursion_depth = saved_recursion;
                            return Err(RuntimeError::InvalidOperation(
                                "break outside loop".into(),
                            ));
                        }
                        Err(RuntimeError::ContinueSignal) => {
                            self.recursion_depth = saved_recursion;
                            return Err(RuntimeError::InvalidOperation(
                                "continue outside loop".into(),
                            ));
                        }
                        Err(e) => {
                            self.recursion_depth = saved_recursion;
                            return Err(e);
                        }
                    }
                }

                self.recursion_depth = saved_recursion;
                Ok(result)
            }
            _ => Err(RuntimeError::InvalidOperation(
                "Value is not callable".into(),
            )),
        }
    }
}
