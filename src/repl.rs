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

//! REPL - Read-Eval-Print Loop

use crate::utils::diagnostics;
use colored::*;
use rustyline::config::Configurer;
use rustyline::{DefaultEditor, EditMode, error::ReadlineError};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🦀 CRISP v0.1.3 - 2026".cyan().bold());
    println!(
        "{}",
        "Creative Rust Implemented Scripting Paradigm".dimmed()
    );
    println!("{}", "Perl-inspired · Rust-powered\n".dimmed());
    println!("Type 'exit' to quit, 'help' for commands");
    println!("Type 'debug verbose' to enable debug, 'debug off' to disable\n");

    let mut rl = DefaultEditor::new()?;
    rl.set_edit_mode(EditMode::Emacs);

    let history_path = format!(
        "{}/.crisp_history",
        std::env::var("HOME").unwrap_or(".".into())
    );
    let _ = rl.load_history(&history_path);

    let mut interpreter = crate::eval::Interpreter::new();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() {
            format!("{} ", ">>>".green())
        } else {
            format!("{} ", "...".yellow())
        };

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();

                match line {
                    "exit" | "quit" | "exit()" => break,
                    "clear" | "cls" => {
                        buffer.clear();
                        std::process::Command::new("clear").status().ok();
                        continue;
                    }
                    "debug on" => {
                        diagnostics::set_debug_level(diagnostics::DebugLevel::Normal);
                        println!("🐛 Debug mode ON (Normal)");
                        continue;
                    }
                    "debug verbose" => {
                        diagnostics::set_debug_level(diagnostics::DebugLevel::Verbose);
                        println!("🐛 Debug mode ON (Verbose - all output)");
                        continue;
                    }
                    "debug off" => {
                        diagnostics::set_debug_level(diagnostics::DebugLevel::Off);
                        println!("🐛 Debug mode OFF");
                        continue;
                    }
                    "help" => {
                        println!("\n{}", "CRISP REPL Commands:".yellow().bold());
                        println!("  {}", "help     Show this help".dimmed());
                        println!("  {}", "exit     Exit REPL".dimmed());
                        println!("  {}", "clear    Clear screen".dimmed());
                        println!("  {}", "debug verbose  Enable debug output".dimmed());
                        println!("  {}", "debug off Disable debug output".dimmed());
                        println!();
                        continue;
                    }
                    "" => {
                        if !buffer.is_empty() {
                            continue;
                        }
                        continue;
                    }
                    _ => {}
                }

                buffer.push_str(line);
                buffer.push('\n');

                if is_complete(&buffer) {
                    match execute_code(&mut interpreter, &buffer) {
                        Ok(Some(value)) => println!("=> {}", value.to_string().cyan()),
                        Ok(None) => {}
                        Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                    }
                    buffer.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                buffer.clear();
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_path).ok();
    Ok(())
}

/// Checks if the input appears complete – supports multi‑line blocks like try/catch/finally.
fn is_complete(code: &str) -> bool {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return false;
    }
    // If it ends with a semicolon or closing brace, assume it's complete.
    // Additional heuristic: if there are unbalanced opening braces, it's incomplete,
    // even if it ends with '}' (e.g. a block inside an unfinished outer block).
    let opens = trimmed.matches('{').count();
    let closes = trimmed.matches('}').count();
    if opens > closes {
        return false; // still inside a block
    }
    // Accept if ends with ; or }
    trimmed.ends_with(';') || trimmed.ends_with('}')
}

fn execute_code(
    interpreter: &mut crate::eval::Interpreter,
    code: &str,
) -> Result<Option<crate::value::Value>, String> {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);
    let program = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    match interpreter.eval(&program) {
        Ok(value) => {
            if matches!(value, crate::value::Value::Null) {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        Err(e) => {
            // ExitSignal must terminate the process — it must NOT be caught
            // and printed as a regular error in the REPL.
            if let crate::eval::RuntimeError::ExitSignal(code) = &e {
                std::process::exit(*code);
            }
            Err(e.to_string())
        }
    }
}
