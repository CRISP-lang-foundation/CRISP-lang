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

//! Diagnostic tools for CRISP

use colored::*;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Debug levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugLevel {
    Off = 0,
    Minimal = 1, // Only errors and warnings
    Normal = 2,  // Basic info
    Verbose = 3, // Everything
}

static DEBUG_LEVEL: AtomicUsize = AtomicUsize::new(0);
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_debug_level(level: DebugLevel) {
    let val = level as usize;
    DEBUG_LEVEL.store(val, Ordering::Relaxed);
    DEBUG_MODE.store(val > 0, Ordering::Relaxed);
}

pub fn get_debug_level() -> DebugLevel {
    match DEBUG_LEVEL.load(Ordering::Relaxed) {
        0 => DebugLevel::Off,
        1 => DebugLevel::Minimal,
        2 => DebugLevel::Normal,
        _ => DebugLevel::Verbose,
    }
}

pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
    if enabled {
        if get_debug_level() == DebugLevel::Off {
            set_debug_level(DebugLevel::Normal);
        }
    } else {
        set_debug_level(DebugLevel::Off);
    }
}

pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

pub fn should_log_debug() -> bool {
    get_debug_level() >= DebugLevel::Verbose
}

pub fn should_log_info() -> bool {
    get_debug_level() >= DebugLevel::Normal
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
    Debug,
}

impl Severity {
    pub fn color(&self) -> colored::Color {
        match self {
            Severity::Error => colored::Color::Red,
            Severity::Warning => colored::Color::Yellow,
            Severity::Info => colored::Color::Cyan,
            Severity::Hint => colored::Color::Green,
            Severity::Debug => colored::Color::BrightBlack,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
            Severity::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub hints: Vec<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: &str) -> Self {
        Diagnostic {
            severity,
            message: message.to_string(),
            span: None,
            hints: Vec::new(),
        }
    }

    pub fn error(message: &str) -> Self {
        Diagnostic::new(Severity::Error, message)
    }

    pub fn warning(message: &str) -> Self {
        Diagnostic::new(Severity::Warning, message)
    }

    pub fn info(message: &str) -> Self {
        Diagnostic::new(Severity::Info, message)
    }

    pub fn hint(message: &str) -> Self {
        Diagnostic::new(Severity::Hint, message)
    }

    pub fn debug(message: &str) -> Self {
        Diagnostic::new(Severity::Debug, message)
    }

    pub fn with_span(mut self, start: usize, end: usize, line: usize, column: usize) -> Self {
        self.span = Some(Span {
            start,
            end,
            line,
            column,
        });
        self
    }

    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hints.push(hint.to_string());
        self
    }

    pub fn with_hints(mut self, hints: Vec<&str>) -> Self {
        self.hints.extend(hints.iter().map(|s| s.to_string()));
        self
    }

    pub fn print(&self, source: Option<&str>) {
        let severity_str = format!("[{}]", self.severity.as_str().to_uppercase());
        let colored_severity = severity_str.color(self.severity.color());

        if self.severity == Severity::Debug && !is_debug_mode() {
            return;
        }

        if self.severity == Severity::Debug {
            println!("{} {}", colored_severity.dimmed(), self.message.dimmed());
            return;
        }

        if let Some(span) = &self.span {
            println!(
                "{} {} at line {}, column {}",
                colored_severity.bold(),
                self.message,
                span.line,
                span.column
            );

            if let Some(source) = source {
                let lines: Vec<&str> = source.lines().collect();
                if span.line <= lines.len() {
                    let line_num = span.line;
                    let line = lines[line_num - 1];
                    println!(" {} | {}", line_num, line);

                    let mut indicator = " ".repeat(span.column + 4);
                    indicator.push_str("^".repeat(span.end - span.start).as_str());
                    println!("{}", indicator.red());
                }
            }
        } else {
            println!("{} {}", colored_severity.bold(), self.message);
        }

        for hint in &self.hints {
            println!("  {} {}", "note:".dimmed(), hint.dimmed());
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.severity.as_str(), self.message)
    }
}

// --- Diagnostics collector (global) ---

pub struct DiagnosticsCollector {
    diagnostics: Vec<Diagnostic>,
    debug_enabled: bool,
}

impl DiagnosticsCollector {
    pub fn new() -> Self {
        DiagnosticsCollector {
            diagnostics: Vec::new(),
            debug_enabled: false,
        }
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug_enabled = enabled;
        self
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        // Debug messages are stored only when debug is enabled
        if diagnostic.severity == Severity::Debug && !self.debug_enabled {
            return;
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, message: &str) {
        self.push(Diagnostic::error(message));
    }

    pub fn warning(&mut self, message: &str) {
        self.push(Diagnostic::warning(message));
    }

    pub fn info(&mut self, message: &str) {
        self.push(Diagnostic::info(message));
    }

    pub fn hint(&mut self, message: &str) {
        self.push(Diagnostic::hint(message));
    }

    pub fn debug(&mut self, message: &str) {
        self.push(Diagnostic::debug(message));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn print_all(&self, source: Option<&str>) {
        if self.diagnostics.is_empty() {
            return;
        }

        let mut sorted = self.diagnostics.clone();
        sorted.sort_by_key(|d| match d.severity {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Info => 2,
            Severity::Hint => 3,
            Severity::Debug => 4,
        });

        for diag in sorted {
            diag.print(source);
            println!();
        }
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

// --- Global diagnostic logger ---

thread_local! {
    static DIAGNOSTICS: RefCell<Option<Rc<RefCell<DiagnosticsCollector>>>> = RefCell::new(None);
}

pub fn init_diagnostics(debug: bool) {
    DIAGNOSTICS.with(|cell| {
        let collector = DiagnosticsCollector::new().with_debug(debug);
        *cell.borrow_mut() = Some(Rc::new(RefCell::new(collector)));
    });
    set_debug_mode(debug);
}

pub fn get_diagnostics() -> Option<Rc<RefCell<DiagnosticsCollector>>> {
    DIAGNOSTICS.with(|cell| cell.borrow().clone())
}

pub fn log(severity: Severity, message: &str) {
    if let Some(collector) = get_diagnostics() {
        collector
            .borrow_mut()
            .push(Diagnostic::new(severity, message));
    }
}

pub fn log_error(message: &str) {
    log(Severity::Error, message);
}

pub fn log_warning(message: &str) {
    log(Severity::Warning, message);
}

pub fn log_info(message: &str) {
    log(Severity::Info, message);
}

pub fn log_hint(message: &str) {
    log(Severity::Hint, message);
}

pub fn log_debug(message: &str) {
    if is_debug_mode() {
        log(Severity::Debug, message);
    }
}

pub fn print_diagnostics(source: Option<&str>) {
    if let Some(collector) = get_diagnostics() {
        collector.borrow().print_all(source);
    }
}

pub fn has_errors() -> bool {
    if let Some(collector) = get_diagnostics() {
        collector.borrow().has_errors()
    } else {
        false
    }
}

pub fn clear_diagnostics() {
    if let Some(collector) = get_diagnostics() {
        collector.borrow_mut().clear();
    }
}

// --- Macro for simple logging ---

#[macro_export]
macro_rules! diag {
    (error: $($arg:tt)*) => {{
        $crate::utils::diagnostics::log_error(&format!($($arg)*));
    }};
    (warning: $($arg:tt)*) => {{
        $crate::utils::diagnostics::log_warning(&format!($($arg)*));
    }};
    (info: $($arg:tt)*) => {{
        $crate::utils::diagnostics::log_info(&format!($($arg)*));
    }};
    (hint: $($arg:tt)*) => {{
        $crate::utils::diagnostics::log_hint(&format!($($arg)*));
    }};
    (debug: $($arg:tt)*) => {{
        $crate::utils::diagnostics::log_debug(&format!($($arg)*));
    }};
}
