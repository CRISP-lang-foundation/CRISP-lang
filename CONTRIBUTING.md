# Contributing to CRISP

Thank you for your interest in contributing to CRISP (Creative Rust Implemented Scripting Paradigm). This document outlines the process for reporting issues, proposing features, and submitting code changes.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Commit Conventions](#commit-conventions)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Issue Reporting](#issue-reporting)
- [Feature Requests](#feature-requests)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

---

## Code of Conduct

Be respectful, constructive, and collaborative. Assume good faith and focus on the technical merits of contributions. Harassment, personal attacks, and disruptive behavior are not tolerated.

---

## Getting Started

### Prerequisites

- **Rust toolchain** (stable) installed via [rustup](https://rustup.rs)
- **Git** for version control
- **Unix-like OS** (Linux or macOS; Windows support is planned)

### Setup

```bash
git clone https://github.com/user/crisp.git
cd crisp
cargo build
cargo test --lib
```

### Run CRISP

```bash
cargo run                    # REPL
cargo run -- script.csp      # Execute a script
cargo run -- -e "say 42"     # One-liner
cargo run -- --ast script.csp # Print AST
```

---

## Project Structure

```
crisp/
  src/
    main.rs          # Entry point, CLI parsing, REPL
    lexer.rs         # Lexer (token stream from source)
    parser.rs        # Pratt parser (token stream to AST)
    ast.rs           # AST node definitions (Expr, Stmt, Value)
    interpreter.rs   # Tree-walking interpreter
    environment.rs   # Environment (scoped variable storage)
    values.rs        # Value enum and native function implementations
    error.rs         # RuntimeError enum and error handling
    posix.rs         # POSIX module (opt-in via `use posix;`)
  tests/
    *.csp            # Integration test scripts
  docs/
    crisp_book.tex   # Companion book (LaTeX)
    crisp_book.pdf   # Rendered book
  README.md
  CONTRIBUTING.md
  LICENSE
  Cargo.toml
```

### Key Design Decisions

- **Tree-walking interpreter:** No bytecode VM yet. The interpreter walks the AST directly at runtime.
- **Pratt parser:** Operator precedence is encoded in binding powers, not grammar rules.
- **Reference-counted values:** Arrays, hashes, and strings use `Rc<RefCell<...>>` for shared ownership and interior mutability.
- **Sentinel errors:** `return`, `break`, and `continue` are implemented as internal `RuntimeError` variants that unwind the call stack.
- **Namespaced POSIX:** System calls live under the `posix` hash variable, loaded with `use posix;`.
- **Lexical scoping:** Each function captures its definition environment via `Rc<RefCell<Environment>>`.

---


## Development Workflow

### Branching Strategy

CRISP uses a multi-branch workflow to keep development organized and stable:

- **main**: Always stable and production-ready. Only thoroughly tested changes are merged here.
- **stage**: Used for integrating, testing, and reviewing new features before they reach `main`. All pull requests target this branch.
- **feat/*** (feature branches): Created from `stage` for each new feature or bugfix. Use clear, descriptive names.

### How to Contribute

1. **Update your local repository:**
   ```bash
   git checkout stage
   git pull
   ```

2. **Create a new feature branch from `stage`:**
   ```bash
   git checkout -b feat/describe-feature
   ```

   Branch naming conventions:
   - `feat/describe-feature` — new functionality
   - `fix/describe-fix` — bug corrections
   - `docs/topic` — documentation updates
   - `refactor/describe-change` — code restructuring
   - `chore/describe-task` — build, CI, tooling
   - `test/describe-tests` — test additions or updates

   Keep branch names short and descriptive. Use lowercase with hyphens.

3. **Work on your feature or fix.** Commit changes following the [commit conventions](#commit-conventions).

4. **Run tests before pushing:**
   ```bash
   cargo test --lib
   cargo clippy
   cargo fmt -- --check
   ```

5. **Push your branch and open a Pull Request to `stage`:**
   ```bash
   git push -u origin feat/describe-feature
   ```
   Then open a Pull Request targeting the `stage` branch.

6. **After review and testing**, your changes will be merged into `stage`. Periodically, stable changes from `stage` are merged into `main` for release.

7. **Delete feature branches after merging** to keep the repository tidy:
   ```bash
   git branch -d feat/describe-feature
   git push origin --delete feat/describe-feature
   ```

### Visual Overview

```
main    ★───────⋆───────⋆───────⋆  (stable releases)
         ↑       ↑       ↑
stage  ──┼───●───┼───●───┼───●──  (integration)
         ↑   ↑       ↑
feat/*  ─┘   └─┘     ─┘            (feature branches)
```

### Why This Strategy?

- **main stays stable.** Users cloning `main` always get working code.
- **stage catches issues.** Features are tested together before reaching `main`.
- **Feature branches keep work isolated.** No half-finished features on shared branches.
- **Clean history.** Squash-merging feature branches into `stage` keeps the history linear and readable.

---

## Commit Conventions

CRISP uses **Conventional Commits** for all changes:

```
<type>(<scope>): <description>
```

### Types

| Type | Usage |
|------|-------|
| `feat` | New feature or language capability |
| `fix` | Bug fix |
| `docs` | Documentation changes (book, README, comments) |
| `style` | Formatting, code style (whitespace, semicolons) |
| `refactor` | Code restructuring without behavior change |
| `test` | Adding or updating tests |
| `chore` | Build, CI, tooling, dependencies |
| `perf` | Performance improvements |

### Scopes

| Scope | Area |
|-------|------|
| `lexer` | Lexer / tokenization |
| `parser` | Parser / AST construction |
| `interpreter` | Tree-walking interpreter |
| `stdlib` | Standard library functions |
| `posix` | POSIX module |
| `error` | Error handling (try/catch/finally, sentinel errors) |
| `repl` | Interactive REPL |
| `cli` | Command-line interface |
| `values` | Value type system |
| `env` | Environment / scoping |
| `regex` | Regular expression support |
| `docs` | Documentation and the companion book |

### Examples

```
feat(interpreter): add try/catch/finally error handling
fix(parser): correct precedence of ternary operator
docs(book): update POSIX function reference table
style(lexer): format with rustfmt
refactor(values): extract native function registration
test(posix): add integration tests for fork/waitpid
chore(deps): update logos to 0.14
perf(interpreter): cache regex compilation results
```

---

## Code Style

### Rust

- **Format with `rustfmt`** before committing:
  ```bash
  cargo fmt
  ```
- **Lint with `clippy`** and fix warnings:
  ```bash
  cargo clippy -- -D warnings
  ```
- Prefer `match` over `if let` chains for exhaustive handling.
- Use descriptive variable names; avoid single-letter names except in iterators and closures.
- Document public functions and types with doc comments (`///`).
- Keep functions under 50 lines where possible.

### CRISP (Test Scripts)

- Use `snake_case` for variables and function names.
- Indent with 4 spaces.
- Use sigils consistently within a file (either all with or all without).
- Include comments (`#`) explaining what each test verifies.

---

## Testing

### Running Tests

```bash
cargo test --lib                  # All tests
cargo test --test <name>    # Specific integration test
cargo test <unit_test_name> # Specific unit test
```

### Writing Tests

**Unit tests** go in the relevant source file (e.g., `#[cfg(test)]` in `parser.rs`).

**Integration tests** go in the `tests/` directory as `.csp` scripts with an accompanying Rust test file.

Each PR that adds or changes behavior must include corresponding tests:
- New language features need integration tests covering both valid and error cases.
- Bug fixes need a regression test that fails before the fix and passes after.
- Refactors need existing tests to continue passing with no changes.

---

## Documentation

### Companion Book

The CRISP book (`docs/crisp_book.tex`) is the official language reference. When adding or changing features:

1. Update the relevant section in `crisp_book.tex`.
2. Verify the example code compiles with the current interpreter.
3. Rebuild the PDF:
   ```bash
   cd docs
   xelatex crisp_book.tex
   xelatex crisp_book.tex  # second pass for TOC
   ```

### Code Documentation

- All public items in the Rust source must have doc comments.
- Use `///` for public API documentation.
- Include code examples in doc comments where helpful.
- Document error conditions, panics, and edge cases.

### README

Update `README.md` when:
- Adding a new major feature
- Changing the build/install process
- Adding new execution modes or CLI flags

---

## Issue Reporting

### Bug Reports

Include:
1. **CRISP version** (`crisp --version`)
2. **Operating system and Rust version** (`rustc --version`)
3. **Minimal reproduction script** (as small as possible)
4. **Expected behavior** vs. **actual behavior**
5. **Error output** (full stack trace if applicable)

### Example

````markdown
## Bug: Division by zero in try/catch produces unexpected null

**Version:** 0.1.0
**OS:** Fedora 43, Rust 1.95.0

**Reproduction:**
```crisp
try {
    let x = 1 / 0;
} catch e {
    say type(e);
}
```

**Expected:** Prints "string" (the error message)
**Actual:** Prints "null"
````

---

## Feature Requests

Before implementing a new feature:

1. **Open an issue** describing the feature and its use case.
2. **Discuss** with maintainers to ensure alignment with the project direction.
3. **Agree on scope** before writing code.

Feature requests should address:
- What problem does it solve?
- How does it fit with CRISP's design philosophy (Perl expressiveness + Rust safety)?
- Is it compatible with the tree-walking interpreter model?
- What is the impact on the language surface area?

---

## Pull Request Process

1. **Keep PRs focused.** One feature or fix per PR.
2. **Update the companion book** if the change affects language behavior.
3. **Add tests** as described above.
4. **Ensure CI passes** (build, tests, clippy, fmt).
5. **Write a clear PR description:**
   - What changed and why
   - How it was tested
   - Any breaking changes or migration notes
6. **Request review** from a maintainer.
7. **Squash merge** after approval (commit message follows conventional commit format).

### Review Criteria

- Does the code follow the project's design patterns?
- Are edge cases handled (null values, type mismatches, error propagation)?
- Do existing tests still pass?
- Is the change backwards-compatible? If not, is the breakage documented?
- Is the companion book updated?
- Does it introduce new dependencies? If so, are they justified?

---

## Release Process

Releases are tagged on `main` with semantic versioning (`v0.1.0`, `v0.1.1`, `v0.2.0`).

1. **Update version** in `Cargo.toml`.
2. **Update the book** title page version badge.
3. **Generate release notes** from conventional commit history.
4. **Tag and push:**
   ```bash
   git tag -a v0.2.0 -m "v0.2.0: <brief summary>"
   git push origin v0.2.0
   ```
5. **Build release binary:**
   ```bash
   cargo build --release
   ```

---

## Questions?

Open an issue with the label `question` or reach out to the maintainers directly. We welcome contributors of all experience levels and are happy to help you get started.
