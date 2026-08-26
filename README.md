# 🦀 CRISP — Creative Rust Implemented Scripting Paradigm

> *Perl's expressiveness · Python's clarity · Rust's safety*

[![Rust](https://img.shields.io/badge/Rust-1.90+-orange.svg)](https://www.rust-lang.org)
[![Top Language](https://img.shields.io/github/languages/top/Peter-L-SVK/CRISP-lang)](https://github.com/Peter-L-SVK/CRISP-lang)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-green.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What is CRISP?

**CRISP** is a scripting language that combines the **expressiveness of Perl 5** with the **safety and performance of Rust** — plus the best ideas from Python, Perl, Rust, and C. Write powerful scripts with familiar syntax, run them on a Rust-powered interpreter that guarantees memory safety and blazing speed.

For a deeper dive, read the ebook avaliable in pdf, also as source code - [CRISP Book](https://github.com/CRISP-lang-foundation/CRISP-book).

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/Peter-L-SVK/CRISP-lang.git
cd CRISP-lang

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

---

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

### Comments

CRISP supports multiple comment styles for flexibility:

```crisp
# Perl-style line comment (starts with #)
// C++-style line comment (starts with //)
/* C-style block comment
   can span multiple lines */

// Comment for the following item
fn example() { /* ... */ }

```

| Style | Syntax | Purpose |
|-------|--------|---------|
| Perl-style | `# comment` | Line comments |
| C++-style | `// comment` | Line comments |
| C-style | `/* comment */` | Block comments (multi-line) |

All comments are ignored by the interpreter.
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

### Class System (Python/Perl-like OOP)

```crisp
# Define a class
class Point {
    fn new(x, y) {
        self.x = x;
        self.y = y;
    }

    fn move_to(x, y) {
        self.x = x;
        self.y = y;
    }

    fn distance() {
        return sqrt(pow(self.x, 2) + pow(self.y, 2));
    }
}

# Instantiate
let p = new Point(3, 4);
say p.x;            # → 3
say p.y;            # → 4
say p.distance();   # → 5

p.move_to(6, 8);
say p.distance();   # → 10

# Inheritance
class Point3D extends Point {
    fn new(x, y, z) {
        super(x, y);       # call parent constructor
        self.z = z;
    }

    fn distance() {
        let d2d = super.distance();
        return sqrt(pow(d2d, 2) + pow(self.z, 2));
    }
}

let p3 = new Point3D(3, 4, 12);
say p3.distance();  # → 13

# Static methods
class Math {
    static fn square(x) { return x * x; }
}
say Math.square(5);  # → 25

# Auto-instantiation: Class.method() creates an object and calls method
say Point.new(1, 2).distance();  # → 2.236...
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

# Functional pipeline: zip → map → filter → take
let @a = [3, 1, 4, 1, 5, 9, 2, 6];
let @b = [2, 7, 1, 8, 2, 8, 1, 8];
let @pairs = @a.zip(@b)
    .map(|pair| => pair[0] * pair[1])
    .filter(|n| => n > 5)
    .take(4);
say @pairs;  # → [6, 7, 8, 10]

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
say person.name;    # → John
say person.age;     # → 30

# Nested structures
let company = {
    name => "CRISP Inc",
    address => {
        city => "Prague",
        country => "Czech Republic"
    }
};
say company.address.city;  # → Prague
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

### JSON Serialization

```crisp
# to_json — serialize any CRISP value to JSON
let data = {
    name => "CRISP",
    version => "0.1.5",
    features => ["oop", "regex", "json"]
};
say to_json(data);
# → {"name":"CRISP","version":"0.1.5","features":["oop","regex","json"]}

# to_json_pretty — pretty-printed JSON
say to_json_pretty(data);

# from_json — parse JSON string to CRISP value
let parsed = from_json('{"x": 10, "y": 20}');
say parsed.x;           # → 10
say parsed.y;           # → 20

# json_valid — check if a string is valid JSON
say json_valid('{"a": 1}');     # → true
say json_valid('not json');     # → false
```

### Regex

```crisp
let text = "hello world, hello CRISP";

# regex_match — check if pattern matches
say regex_match(text, "CRISP");         # → true
say regex_match(text, "^hello");        # → true

# regex_replace — replace all occurrences
say regex_replace(text, "hello", "hi"); # → hi world, hi CRISP

# regex_split — split by regex
say regex_split("a,b;c:d", "[,;:]");    # → [a, b, c, d]

# regex_find_all — find all matches
say regex_find_all(text, "\\w+");       # → [hello, world, hello, CRISP]

# regex_capture — capture groups
let caps = regex_capture("John, 30", "(\w+), (\d+)");
say caps[1];  # → John
say caps[2];  # → 30
```

### Networking

```crisp
# TCP client
let sock = tcp_connect("example.com:80");
tcp_write(sock, "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
let response = tcp_read(sock, 4096);
tcp_close(sock);
say response;

# TCP server
let server = tcp_listen("127.0.0.1:3000");
let conn = tcp_accept(server);
say conn.addr;              # → 127.0.0.1:54321
tcp_write(conn.stream, "Hello from CRISP!\n");
tcp_close(conn.stream);

# HTTP GET shortcut
let html = http_get("http://example.com/");
say html;
```

### Time

```crisp
# Current timestamp
say time();             # → 1717000000 (Unix timestamp)
say timestamp();        # → 1717000000123 (milliseconds)

# Current date/time
say datetime();         # → 2026-05-29 14:30:00
say datetime_utc();     # → 2026-05-29 12:30:00 UTC

# Format and parse
say strftime("%Y-%m-%d %H:%M:%S", time());
let ts = strptime("2026-05-29 14:30:00", "%Y-%m-%d %H:%M:%S");

# Sleep
sleep(1);               # seconds
sleep_ms(500);          # milliseconds
```

### Crypto

```crisp
# Hashing
say md5("hello");               # → 5d41402abc4b2a76b9719d911017c592
say sha1("hello");              # → aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
say sha256("hello");            # → 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c...
say sha512("hello");

# Base64
say base64_encode("CRISP");     # → Q1JJU1A=
say base64_decode("Q1JJU1A=");  # → CRISP

# Random bytes
say random_bytes(16);           # → (16 random bytes as hex)
```

### Process Management

```crisp
# Run a command
system("ls", "-la");

# Capture output
let output = qx("date");
say output;

# Execute (replace current process)
exec("vim", "hello.crisp");

# Run via shell
shell("echo hello > /tmp/test.txt");

# Get process ID
say pid();
```

### Testing

```crisp
# Assertions
assert_eq(2 + 2, 4);
assert_eq(sqrt(9), 3, "sqrt(9) should be 3");
assert_ne(1, 2);
assert_true(1 < 5);
assert_false(0 > 10);

# Assert function throws
fn bad() { die "oops"; }
assert_throws(bad);

# Named test
test("my_test", | | => {
    assert_eq(pow(2, 3), 8);
    assert_true("hello".len() == 5);
    say "All assertions passed!";
});
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

---

## Project Structure

```
CRISP-lang/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI arguments
│   ├── repl.rs              # REPL interface
│   ├── lexer/               # Tokenizer
│   ├── parser/              # AST builder
│   ├── eval/                # Interpreter
│   ├── value/               # Value system
│   ├── stdlib/              # Standard library
│   │   ├── mod.rs           # Module registry
│   │   ├── io.rs            # Console I/O
│   │   ├── math.rs          # Math functions
│   │   ├── collections.rs   # map, grep, sort, push, pop, etc.
│   │   ├── json.rs          # JSON serialization
│   │   ├── regex.rs         # Regex engine
│   │   ├── network.rs       # TCP sockets, HTTP
│   │   ├── time.rs          # Time functions
│   │   ├── crypto.rs        # Hashing, base64
│   │   ├── process.rs       # system, exec, qx, shell
│   │   ├── testing.rs       # Unit testing
│   │   ├── filesystem.rs    # File system operations
│   │   └── posix/           # POSIX system calls (opt-in via use posix)
│   └── utils/               # Utilities
├── examples/                # Example scripts
│   ├── hello.crisp
│   ├── quadratic.csp
│   ├── functional.csp
│   └── ...
├── tests/                   # Integration tests
└── docs/                    # Documentation
```

---

## Standard Library

### Console I/O
- `print(args...)` — Output without newline
- `say(args...)` — Output with newline
- `warn(args...)` — Output to STDERR
- `readline(prompt?)` → String — Read a line from stdin
- `read(prompt?)` → Int | Float | Str — Read line with auto-detect type
- `input(prompt?)` → String — Python-compatible alias for readline

### Error Handling
- `die(msg)` — Catchable user error
- `throw(msg)` — Alias for die (catchable)
- `assert(cond, msg?)` — Testing assertion

### Type Conversion
- `int(value)` → Int | Null — Parse string to integer
- `float(value)` → Float | Null — Parse string to float
- `type(value)` → String — Returns type name
- `len(value)` → Int — Length of string, array, or hash

### Array Methods (via `.method()` syntax)
- `.len()` — Array length
- `.contains(val)` — Check if array contains value
- `.push(val)` — Append element(s), returns new length
- `.pop()` — Remove and return last element
- `.shift()` — Remove and return first element
- `.unshift(val)` — Prepend element(s)
- `.map(\|x\| => expr)` — Transform each element
- `.filter(\|x\| => expr)` — Keep matching elements
- `.grep(\|x\| => expr)` — Alias for filter
- `.zip(other_array)` — Pair elements from two arrays
- `.take(n)` — Take first n elements
- `.join(sep)` — Join elements with separator
- `.keys()` — Hash keys (on hashes)
- `.values()` — Hash values (on hashes)

### Standalone Array Functions
- `map(func, arr)` — Transform array
- `grep(func, arr)` — Filter array
- `sort(arr, comparator?)` — Sort array
- `push(arr, val)` — Append to array
- `pop(arr)` — Remove last element
- `shift(arr)` — Remove first element
- `unshift(arr, val)` — Prepend to array
- `join(arr, sep)` — Join array into string

### String Methods
- `.len()` — String length
- `.contains(substr)` — Check if string contains substring
- `.split(delim)` — Split into array

### Math & Random
- `sqrt(x)`, `pow(base, exp)`, `abs(x)`, `min(a, b)`, `max(a, b)`
- `rand(max?)` → Int — Random integer 0..max-1
- Constants: `PI`, `E`, `TAU`

### JSON
- `to_json(value)` → String — Serialize to JSON
- `to_json_pretty(value)` → String — Pretty-printed JSON
- `from_json(string)` → Value — Parse JSON string
- `json_valid(string)` → Bool — Check if valid JSON

### Regex
- `regex_match(text, pattern)` → Bool
- `regex_replace(text, pattern, replacement)` → String
- `regex_split(text, pattern)` → Array
- `regex_find_all(text, pattern)` → Array
- `regex_capture(text, pattern)` → Array of captures

### Networking
- `tcp_connect(addr)` → Int — Connect to TCP server, returns socket ID
- `tcp_listen(addr)` → Int — Start TCP server, returns listener ID
- `tcp_accept(listener_id)` → Hash — Accept connection, returns `{stream, addr}`
- `tcp_read(socket_id, length)` → String — Read from socket
- `tcp_write(socket_id, data)` → Int — Write to socket, returns bytes written
- `tcp_close(socket_id)` → Bool — Close socket
- `http_get(url)` → String — Simple HTTP GET

### Time
- `time()` → Int — Unix timestamp (seconds)
- `timestamp()` → Int — Unix timestamp (milliseconds)
- `datetime()` → String — Current local date/time
- `datetime_utc()` → String — Current UTC date/time
- `strftime(format, timestamp)` → String — Format timestamp
- `strptime(string, format)` → Int — Parse string to timestamp
- `sleep(seconds)` — Sleep for seconds
- `sleep_ms(milliseconds)` — Sleep for milliseconds

### Crypto
- `md5(string)` → String — MD5 hash
- `sha1(string)` → String — SHA-1 hash
- `sha256(string)` → String — SHA-256 hash
- `sha512(string)` → String — SHA-512 hash
- `base64_encode(string)` → String — Base64 encode
- `base64_decode(string)` → String — Base64 decode
- `random_bytes(length)` → String — Random bytes as hex

### Process Management
- `system(cmd, args...)` → Int — Run command, return exit code
- `exec(cmd, args...)` — Replace current process
- `qx(cmd, args...)` → String — Run command, capture stdout
- `shell(cmd)` → Int — Run via shell
- `pid()` → Int — Current process ID

### Testing
- `test(name, func)` — Declare and run a named test
- `assert_eq(a, b, msg?)` — Assert equality
- `assert_ne(a, b, msg?)` — Assert inequality
- `assert_true(val, msg?)` — Assert truthy
- `assert_false(val, msg?)` — Assert falsy
- `assert_throws(func, msg?)` — Assert function throws error
- `test_suite()` — Run all tests

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

---

## Language Design

See [ROADMAP.md](https://github.com/Peter-L-SVK/CRISP-lang/blob/main/ROADMAP.md) for details.

### Perl-inspired Features
- ✅ **Sigil variables** — `$scalar`, `@array`, `%hash`, `&ref`
- ✅ **Regular expressions** — Native `=~`, `s///`, full regex module
- ✅ **Built-in functions** — `map`, `grep`, `push`, `pop`, `system`, `qx`
- ✅ **Method chaining** — `@arr.map(...).filter(...).take(3)`
- ✅ **One-liners** — `crisp -e 'code'`
- ✅ **Comments** — `#` style

### Python-inspired Features
- ✅ **OOP system** — `class`, `extends`, `super()`, `self`, constructors
- ✅ **Type inference** — No explicit types needed
- ✅ **Exception handling** — `try`/`catch`/`finally`
- ✅ **Console I/O** — `input()` alias for `readline()`
- ✅ **Rich standard library** — JSON, regex, networking, crypto, time, testing

### Rust-inspired Features
- ✅ **Memory safety** — No segfaults, no dangling pointers
- ✅ **Lambda functions** — `|x| => x * 2`
- ✅ **Expression-oriented** — Everything returns a value
- ✅ **Pattern matching** — `match` with `where` guards

### Unique CRISP Features
- ✅ **Class.method() auto-instantiation** — Any method on a class creates an object
- ✅ **Method dispatch** — `@arr.push(4)`, `"str".split(",")`, `obj.method()`
- ✅ **Three-way comparison** — `<=>` spaceship operator
- ✅ **Ternary operator** — `cond ? then : else`
- ✅ **POSIX system calls** — Full Unix/Linux API via `use posix`
- ✅ **Hash field access** — `person.name` dot notation
- ✅ **Automatic hash keys** — `{ name => "John" }` without quoting keys
- ✅ **References** — `\expr` creates, `^ref` dereferences
- ✅ **Array/string slicing** — `arr[1..3]`, `str[0..4]`
- ✅ **Range expressions** — `1..10`, `"a".."z"`

---

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

# Quadratic equation solver
cargo run -- examples/quadratic.csp

# Functional pipeline demo
cargo run -- examples/functional.csp

# Run stdlib demo
cargo run -- examples/stdlib_demo.crisp
```

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](https://github.com/Peter-L-SVK/CRISP-lang/blob/main/CONTRIBUTING.md) for details.

If you wish to express your ideas, feel free to do so in Discussions.

1. Fork the repository
2. Create a feature branch
3. Write code with tests
4. Commit changes
5. Open a Pull Request

### Areas Needing Help
- Bytecode VM implementation
- Module system (user-defined modules, import paths)
- Stack traces for debugging (file, line, backtrace)
- Windows support
- Signal handling (SIGINT, SIGTERM)
- Int ↔ Float autoconversion in arithmetic
- Named arguments in function calls

---

## License

This project is dual-licensed under:

- **MIT License** — see [LICENSE-MIT](LICENSE-MIT) for details
- **Apache License 2.0** — see [LICENSE-APACHE](LICENSE-APACHE) for details

---

*"Keep your code CRISP and clean"* 🦀🐪
