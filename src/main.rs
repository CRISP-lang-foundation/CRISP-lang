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

//! CRISP - Creative Rust Implemented Scripting Paradigm
//!
//! Main entry point

mod cli;
mod eval;
mod lexer;
mod parser;
mod repl;
mod stdlib;
mod utils;
mod value;

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use utils::diagnostics;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    #[arg(short, long)]
    repl: bool,
    #[arg(short = 'e', long)]
    eval: Option<String>,
    #[arg(long)]
    ast: bool,
    #[arg(short, long)]
    debug: bool,
    #[arg(long)]
    test: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            if let Err(e) = run_main() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        })
        .unwrap()
        .join();

    if let Err(e) = result {
        eprintln!("Thread panicked: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.debug {
        diagnostics::init_diagnostics(true);
        diagnostics::set_debug_level(diagnostics::DebugLevel::Normal);
        println!("🐛 Debug mode ON (Normal)");
    } else {
        diagnostics::init_diagnostics(false);
        diagnostics::set_debug_level(diagnostics::DebugLevel::Off);
    }

    if cli.test {
        return run_tests();
    }

    if cli.repl || (cli.file.is_none() && cli.eval.is_none()) {
        return repl::run();
    }

    if let Some(code) = cli.eval {
        return run_code(&code, cli.ast);
    }

    if let Some(file) = cli.file {
        return run_file(&file, cli.ast);
    }

    Ok(())
}

fn run_file(file: &PathBuf, print_ast: bool) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ext) = file.extension() {
        if ext != "csp" && ext != "crisp" {
            eprintln!(
                "⚠Warning: File '{}' has non-standard extension",
                file.display()
            );
            eprintln!("   Expected: .csp or .crisp");
        }
    } else {
        eprintln!("⚠Warning: File '{}' has no extension", file.display());
        eprintln!("   Expected: .csp or .crisp");
    }

    let code = fs::read_to_string(file)?;
    run_code(&code, print_ast)
}

fn run_code(code: &str, print_ast: bool) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    use eval::Interpreter;
    use eval::RuntimeError;
    use lexer::Lexer;
    use parser::Parser;

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);
    let program = parser.parse()?;

    if print_ast {
        println!("{}", "AST:".cyan().bold());
        println!("{:#?}", program);
        return Ok(());
    }

    let mut interpreter = Interpreter::new();
    match interpreter.eval(&program) {
        Ok(result) => {
            if !matches!(result, value::Value::Null) {
                println!("{}", result);
            }
            Ok(())
        }
        Err(e) => {
            if let RuntimeError::ExitSignal(code) = &e {
                std::process::exit(*code);
            }
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn run_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running CRISP tests...");
    Ok(())
}
