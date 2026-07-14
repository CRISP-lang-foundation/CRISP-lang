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

## v0.2.0 — Advanced Features
- [ ] Module system (extend `use` for user-defined modules, import paths)
- [ ] Full regex engine (beyond basic `=~` and `s///`)
- [ ] OOP — Python3/Perl-like class system
- [ ] JSON serialization
- [ ] Network sockets
- [ ] Stack traces for debugging
- [ ] More type coercion (int ↔ float autoconversion)
- [ ] Named arguments in function calls

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
