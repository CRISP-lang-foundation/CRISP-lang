# 🦀 CRISP - Creative Rust Implemented Scripting Paradigm

> *Perl's expressiveness · Rust's safety*

[![Rust](https://img.shields.io/badge/Rust-1.90+-orange.svg)](https://www.rust-lang.org)
[![Top Language](https://img.shields.io/github/languages/top/Peter-L-SVK/pro_audio_config)](https://github.com/Peter-L-SVK/CRISP-lang)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-green.svg)](LICENSE)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What is CRISP?

**CRISP** is a scripting language that combines the **expressiveness of Perl 5** with the **safety and performance of Rust**. Write powerful scripts with Perl-inspired syntax, run them on a Rust-powered interpreter that guarantees memory safety and blazing speed. Best from world of Python, Perl, Rust and C.  

For more details read the ebook. 

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/crisp-lang/crisp.git
cd crisp

# Build
cargo build --release

# Install globally
cargo install --path .
```

### Try It Now

```bash
# REPL mode
crisp

# Run a script
crisp examples/hello.crisp

# One-liner
crisp -e 'say "Hello, CRISP!"'

# Print AST (for debugging)
crisp --ast examples/hello.crisp
```

## Language Examples

### Hello World
```crisp
# Perl-style comments
say "Hello, CRISP!";        # With newline
print "Hello, ";            # Without newline
say "World!";               # Hello, World!

# Variables
let name = "CRISP";
let fruits = ["apple", "banana", "cherry"];
let person = { name => "John", age => 30 };

say "Hello, " + name + "!";        # String concatenation
say fruits;                         # [apple, banana, cherry]
say person;                         # {name: John, age: 30}
```

### Sigil Variables (Perl-style)
```crisp
# $scalar — single value
let $count = 42;
say $count;                     # → 42

# @array — ordered list
let @fruits = ["apple", "banana", "cherry"];
say @fruits;                    # → [apple, banana, cherry]
say @fruits[0];                 # → apple
say @fruits.len();              # → 3

# &reference — function or code ref
let &callback = |x| => x * 2;
say &callback(5);               # → 10
```

### Try/Catch/Finally — Error Handling
```crisp
# Basic try/catch
try {
    let x = 1 / 0;
} catch e {
    say "Error: " + e;          # → Error: Division by zero
};

# throw & die (both catchable)
try {
    throw "something went wrong";
} catch e as err {
    say "Caught: " + err;       # → Caught: something went wrong
};

# try/catch/finally
try {
    say "trying...";
    die "oops";
} catch e {
    say "error: " + e;
} finally {
    say "cleanup always runs";
};

# Nested error handling
try {
    try {
        throw("inner error");
    } catch e {
        say "inner: " + e;
        throw("re-thrown");
    };
} catch e {
    say "outer: " + e;
};
```

### Method Chaining on Arrays
```crisp
let @nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

# push / pop
@nums.push(11);
say @nums.pop();                # → 11

# shift / unshift
say @nums.shift();              # → 1
@nums.unshift(0);

# map — transform
let @doubled = @nums.map(|x| => x * 2);
say @doubled;                   # → [0, 4, 6, 8, 10, 12, 14, 16, 18, 20]

# filter / grep — keep matching
let @evens = @nums.filter(|x| => x % 2 == 0);
say @evens;                     # → [0, 4, 6, 8, 10]

# Chaining methods
let @result = @nums
    .map(|x| => x * 3)
    .filter(|x| => x > 10);
say @result;                    # → [12, 15, 18, 21, 24, 27, 30]

# keys / values on hashes
let %scores = { alice => 10, bob => 20, charlie => 30 };
say %scores.keys();             # → [alice, bob, charlie]
say %scores.values();           # → [10, 20, 30]
```

### Hash Field Access
```crisp
# Automatic string keys — no quotes needed!
let person = { name => "John", age => 30 };

# Field access with dot notation
say person.name;    # John
say person.age;     # 30

# Nested structures
let company = {
    name => "CRISP Inc",
    address => {
        city => "Prague",
        country => "Czech Republic"
    }
};
say company.address.city;  # Prague
```
### Control Flow Keywords

```crisp
# return — exit function with optional value
fn double(x) { return x * 2; }

# break — exit loop early
while true {
    if condition { break; }
}

# continue — skip to next iteration
for i in 0..10 {
    if i % 2 == 0 { continue; }  # skip evens
    say i;                        # 1 3 5 7 9
}

# Flow control signals propagate correctly through try/catch/finally:
while true {
    try {
        break;              # finally still executes
    } finally {
        say "cleanup!";
    }
}
```

### Arithmetic
```crisp
let a = 10;
let b = 20;
say a + b;  # 30
say a * b;  # 200
say a - b;  # -10
say b / a;  # 2
say a ** 3; # 1000 (power)
say b % 3;  # 2   (modulo)
```

### Lambda Functions
```crisp
# Lambda with explicit return
let square = |x| => { return x * x; };
say square(5);  # 25

# Lambda with expression body (implicit return)
let double = |x| => x * 2;
say double(10);  # 20

# Lambda with multiple parameters
let add = |a, b| => a + b;
say add(3, 7);  # 10

# Higher-order functions
let @numbers = [1, 2, 3, 4, 5];
let @doubled = @numbers.map(|n| => n * 2);
say @doubled;  # [2, 4, 6, 8, 10]
```

### Named Functions
```crisp
fn factorial(n) {
    if n <= 1 { return 1; };
    return n * factorial(n - 1);
};

say factorial(5);  # 120
say factorial(10); # 3628800

fn greet(name, greeting) {
    return greeting + ", " + name + "!";
};

say greet("CRISP", "Hello");  # Hello, CRISP!
```

### Loops

```crisp
# While loop with break / continue
let i = 0;
while true {
    print(i);
    i = i + 1;
    if i >= 5 { break; }
}
# Output: 0 1 2 3 4

# For loop over array
let @numbers = [1, 2, 3, 4, 5];
for n in @numbers {
    print(n * n);
}
# Output: 1 4 9 16 25

# Continue skips iteration
for n in 0..10 {
    if n == 3 { continue; }
    if n == 7 { break; }
    say n;
}
# Output: 0 1 2 4 5 6

# For loop over hash
let %ages = { alice => 25, bob => 30 };
for age in %ages {
    say age;
}
# Output: 25 30

# For loop over string
for ch in "CRISP" {
    say ch;
}
# Output: C R I S P
```

### Conditional Statements
```crisp
let age = 18;
if age >= 18 {
    say "Adult";
} else {
    say "Child";
};
# Output: Adult

let score = 85;
if score >= 90 {
    say "A";
} else if score >= 80 {
    say "B";
} else {
    say "C";
};
# Output: B
```

### Match with Guards (Pattern Matching)

```crisp
# Basic match
let x = 42;
match x {
    1  => { say "one"; }
    42 => { say "the answer"; }
    _  => { say "something else"; }
}

# Match with where guards
let guess = read("Your guess: ");

match guess {
    n where type(n) == "int" => {
        say "You entered integer: ", n;
    }
    s where s == "" => {
        say "Empty input.";
    }
    _ => {
        say "Something else: ", guess;
    }
}
```

### Arrays and Hashes
```crisp
# Arrays
let @numbers = [1, 2, 3, 4, 5];
say @numbers[0];     # 1
say @numbers[2];     # 3

# Array slicing
say @numbers[1..3];  # [2, 3, 4]

# Hashes with automatic string keys
let person = { name => "John", age => 30 };
say person.name;    # John (dot notation)
say person.age;     # 30

# Nested structures
let data = {
    users => [
        { name => "Alice", age => 25 },
        { name => "Bob", age => 30 }
    ]
};
say data.users[0].name;  # Alice
```

### Multi-Dimensional Arrays

```crisp
# Nested array literals
let @matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];

# Chained indexing
say @matrix[1][2];        # → 6
say @matrix[0];            # → [1, 2, 3]

# Nested for-loops
for row in @matrix {
    for cell in row {
        say cell;           # → 1 2 3 4 5 6 7 8 9
    }
}

# Contains works on arrays of arrays
let @row = @matrix[0];
say @row.contains(2);       # → true
```

### References
```crisp
# Create a reference
let x = 42;
let r = \x;         # r is a Ref
say type(r);        # → ref

# Dereference
say ^r;             # → 42

# Modify through reference
^x = 99;            # (future: assign through deref)
```

### String Operations
```crisp
let s = "hello,world,crisp";

# Split
let @parts = s.split(",");
say @parts;         # → [hello, world, crisp]

# Join
say @parts.join(" - ");  # → hello - world - crisp

# Concatenation
say "hello" . " " . "world";  # → hello world

# Repeat
say "ha" x 3;       # → hahaha

# Length
say s.len();        # → 19
```

### Console I/O

```crisp
# readline — always returns a String
let name = readline("What's your name? ");
say "Hello, ", name, "!";

# read — auto-detects Int, Float, or String
let age = read("How old are you? ");
say type(age);         # → int, float, or string

# input — Python-compatible alias for readline
let color = input("Favorite color? ");

# EOF (Ctrl+D) returns an empty string
let data = readline();
if data == "" {
    say "Goodbye!";
}
```

### System Programming (POSIX — Unix only)

POSIX is loaded on demand via `use posix;` and lives under a namespace hash:

```crisp
# Load POSIX module
use posix;

# Process management
let pid = posix["fork"]();
if pid == 0 {
    say "Child process: " + posix["getpid"]();
    posix["execvp"]("ls", "-la");
} else {
    say "Parent waiting for child: " + pid;
    let status = posix["waitpid"](pid, 0);
    say "Child exited with: " + status;
}

# File operations
let fd = posix["open"]("./test.txt",
    posix["O_WRONLY"] | posix["O_CREAT"], 0o644);
posix["write"](fd, "Hello POSIX!\n", 13);
posix["close"](fd);

let fd2 = posix["open"]("./test.txt", posix["O_RDONLY"]);
let data = posix["read"](fd2, 100);
posix["close"](fd2);
say data;

# No namespace pollution — console read() still works!
let input = readline("> ");
```

## Project Structure

```
crisp/
├── src/
│   ├── main.rs          # Entry point
│   ├── cli.rs           # CLI arguments
│   ├── repl.rs          # REPL interface
│   ├── lexer/           # Tokenizer
│   ├── parser/          # AST builder
│   ├── eval/            # Interpreter
│   ├── value/           # Value system
│   ├── stdlib/          # Standard library
│   │   ├── io.rs        # Console I/O (print, say, readline, read, input)
│   │   ├── math.rs      # Math functions
│   │   ├── string.rs    # String operations
│   │   ├── collections.rs # map, grep, sort
│   │   ├── filesystem.rs  # File system operations
│   │   └── posix/       # POSIX system calls (opt-in via use posix)
│   └── utils/           # Utilities
├── examples/            # Example scripts
├── tests/              # Integration tests
└── docs/               # Documentation
```

## Standard Library

### Console I/O
- `print(args...)` — Output without newline (supports bare and paren syntax)
- `say(args...)` — Output with newline (supports bare and paren syntax)
- `warn(args...)` — Output to STDERR
- `readline(prompt?)` → String — Read a line from stdin, strips trailing newline
- `read(prompt?)` → Int | Float | Str — Like readline but auto-detects type
- `input(prompt?)` → String — Python-compatible alias for readline

### Error Handling
- `die(msg)` — Catchable user error
- `throw(msg)` — Alias for die (catchable)
- `assert(cond, msg?)` — Testing assertion

### Type Conversion
- `int(value)` → Int | Null — Parse string to integer, returns null on failure
- `float(value)` → Float | Null — Parse string to float, returns null on failure
- `type(value)` → String — Returns type name
- `len(value)` → Int — Length of string, array, or hash

### Array Operations
- `push(arr, val)` — Append to array
- `pop(arr)` — Remove and return last element
- `map(arr, callback)` — Transform array
- `filter(arr, callback)` — Filter array

### Array Methods (via `.method()` syntax)
- `.contains(val)` — Check if array contains value
- `.push(val)` — Append element(s), returns new length
- `.pop()` — Remove and return last element
- `.shift()` — Remove and return first element
- `.unshift(val)` — Prepend element(s)
- `.len()` — Array length
- `.map(|x| => expr)` — Transform each element
- `.filter(|x| => expr)` — Keep matching elements
- `.grep(|x| => expr)` — Alias for filter
- `.join(sep)` — Join elements with separator
- `.keys()` — Hash keys (on hashes)
- `.values()` — Hash values (on hashes)

### String Methods
- `.len()` — String length
- `.contains(substr)` — Check if string contains substring
- `.split(delim)` — Split into array
- `.join(sep)` — Join array into string

### Math & Random
- `rand(max?)` → Int — Random integer 0..max-1 (max defaults to u64::MAX)
- `sqrt()`, `pow()`, `abs()`, `min()`, `max()`
- Constants: `PI`, `E`

### File I/O
- `read_file(path)` — Read entire file
- `write_file(path, content)` — Write to file
- `file_exists(path)` — Check if file exists

### Filesystem Module
- `list_dir()`, `create_dir()`, `remove_dir()`
- `is_dir()`, `is_file()`

### POSIX Module (Unix — opt-in via `use posix;`)
- **Processes**: `posix["fork"]()`, `posix["execvp"]()`, `posix["waitpid"]()`, `posix["spawn"]()`, `posix["getpid"]()`, `posix["getppid"]()`
- **Signals**: `posix["kill"]()`, `posix["alarm"]()`
- **Users**: `posix["getuid"]()`, `posix["geteuid"]()`, `posix["getgid"]()`, `posix["getegid"]()`
- **Files**: `posix["open"]()`, `posix["close"]()`, `posix["read"]()`, `posix["write"]()`, `posix["lseek"]()`
- **Environment**: `posix["getenv"]()`, `posix["setenv"]()`, `posix["environ"]()`

## Development

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

### Examples
```bash
# Run hello world
cargo run -- examples/hello.crisp

# Run stdlib demo
cargo run -- examples/stdlib_demo.crisp
```

## Language Design

### Perl-inspired Features
- ✅ **Sigil variables** — `$scalar`, `@array`, `&ref`
- ✅ **Regular expressions** — Native `=~`, `s///`
- ✅ **Built-in functions** — `map`, `grep`, `push`, `pop`
- ✅ **Method chaining** — `@arr.map(...).filter(...)`
- ✅ **One-liners** — `crisp -e 'code'`
- ✅ **Comments** — `#` style

### Rust-inspired Features
- ✅ **Memory safety** — No segfaults
- ✅ **Lambda functions** — `|x| => x * 2`
- ✅ **Type inference** — No explicit types needed
- ✅ **Expression-oriented** — Everything returns a value

### Unique CRISP Features
- ✅ **Try/Catch/Finally** — `try { } catch e as err { } finally { }`
- ✅ **throw & die** — Catchable error keywords
- ✅ **Method dispatch** — `@arr.push(4)`, `"str".split(",")`
- ✅ **Three-way comparison** — `<=>` spaceship operator
- ✅ **Ternary operator** — `cond ? then : else`
- ✅ **POSIX system calls** — Full Unix/Linux API
- ✅ **Lambda with return** — `{ return x; }`
- ✅ **Hash field access** — `person.name` dot notation
- ✅ **Automatic hash keys** — `{ name => "John" }`
- ✅ **References** — `\expr` creates, `^ref` dereferences
- ✅ **Array/string slicing** — `arr[1..3]`, `str[0..4]`
- ✅ **Range expressions** — `1..10`, `"a".."z"`

## Contributing

We welcome contributions!
See [CONTRIBUTING](https://github.com/Peter-L-SVK/CRISP-lang/blob/main/CONTRIBUTING.md) file for details.  

If you wish to express your Ideas feel free to do so in Discussions.  
  
1. Fork the repository
2. Create a feature branch
3. Write code with tests
4. Commit changes
5. Open a Pull Request

### Areas Needing Help
- Implementing `match` pattern matching
- Bytecode VM optimization
- More POSIX system calls
- Windows support
- GUI framework integration

## License

This project is dual-licensed under:

- **MIT License** - [MIT License](LICENSE-MIT) - see the LICENSE file for details.
- **Apache License 2.0** -  [Apache License 2.0](LICENSE-APACHE) - see the LICENSE file for details.

---

*"Keep your code CRISP and clean"* 🦀🐪

