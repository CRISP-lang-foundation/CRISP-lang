# Roadmap

## v0.1.0 — Foundation
- [x] Lexer with full token support
- [x] Parser with AST generation (Pratt parser)
- [x] Tree-walking interpreter
- [x] Core value system
- [x] Basic standard library
- [x] POSIX system calls (opt-in via `use posix`)
- [x] REPL interface
- [x] CLI with one-liner support
- [x] Named functions (`fn`) with recursion
- [x] Lambda functions with return
- [x] While and For loops with `break`/`continue`
- [x] Variable assignment (`x = expr`)
- [x] Hash and Array data structures
- [x] Conditional statements (if/else/else-if)
- [x] String concatenation
- [x] Full support for binary ops
- [x] Hash field access (dot notation)
- [x] Automatic hash key conversion
- [x] Try/Catch/Finally with `catch e as err` alias
- [x] throw / die keywords
- [x] Method chaining on arrays, hashes, strings
- [x] Sigil variables ($scalar, @array, &ref)
- [x] Array/string slicing and ranges
- [x] References (\ref and ^deref)
- [x] Match with `where` guards (pattern matching)
- [x] Console I/O: `readline`, `read`, `input` with flush semantics
- [x] `rand()` random number generator
- [x] `int()` / `float()` type conversion
- [x] `null` equality comparisons

## v0.1.5 — Polish & Fixes (current)
- [x] Python/Perl-like OOP (classes, constructors, inheritance, super)
- [x] JSON serialization (to_json, from_json, to_json_pretty, json_valid)
- [x] Network sockets (tcp_connect, tcp_listen, tcp_accept, tcp_read/write, http_get)
- [x] Full regex module (match, replace, split, find_all, capture)
- [x] Major bugfixes: brace-depth tracking, method dispatch routing, bool ops
- [x] English-only codebase (all Slovak comments translated)
- [x] New code examples

## v0.2.0 — Advanced Features
- [ ] Module system (extend `use` for user-defined modules, import paths)
- [ ] Stack traces for debugging (file, line, backtrace on errors)
- [ ] More type coercion (int ↔ float autoconversion in arithmetic)
- [ ] Named arguments in function calls
- [ ] File include/require mechanism
- [ ] Signal handling (SIGINT, SIGTERM)
- [ ] Process management (system, exec, qx, pid, shell — already in process.rs)

## v0.3.0 — Performance
- [ ] Bytecode VM
- [ ] JIT compilation
- [ ] Async/Await support
- [ ] Multi-threading

## v1.0.0 — Production Ready
- [ ] Stable language spec
- [ ] Comprehensive standard library
- [ ] Package manager (CRISPAN)
- [ ] Language Server Protocol (LSP)
- [ ] Full documentation
