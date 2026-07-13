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

//! CLI arguments and configuration

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Script file to execute
    pub file: Option<String>,

    /// Run interactive REPL
    #[arg(short, long)]
    pub repl: bool,

    /// Execute one-liner
    #[arg(short = 'e', long)]
    pub eval: Option<String>,

    /// Print AST
    #[arg(long)]
    pub ast: bool,

    /// Verbose mode
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable debugging
    #[arg(long)]
    pub debug: bool,

    /// Run test suite
    #[arg(long)]
    pub test: bool,
}
