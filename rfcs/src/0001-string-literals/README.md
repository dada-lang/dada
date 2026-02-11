# RFC-0001: String Literals

---
status: active
---

## Summary

Define string literal syntax for Dada that makes string interpolation the default behavior, learning from the evolution of string handling in languages like Rust and the success of template literals in JavaScript/TypeScript. String literals in Dada have type `my String`.

## Design tenets

1. **Do what I mean** - Default behavior matches common intent: interpolation enabled, indentation automatically handled
2. **Rust-like syntax** - Use familiar `{}` for interpolation, maintaining consistency with Rust's formatting approach  
3. **Simple escape hatch** - When you need exact control, a single character prefix (`"\`) disables all magic

## Motivation

String manipulation is one of the most common operations in programming. Languages have evolved different approaches:

**Rust's Evolution**: Rust started with C-like static string literals (`"hello"`), requiring explicit formatting macros for variable interpolation:
```rust
let name = "Alice";
let message = format!("Hello, {}!", name);
```

Over time, Rust has added increasingly convenient forms:
- `println!` and other macros for common cases
- Recent discussions about f-strings or similar interpolation syntax
- Recognition that the default (static strings) doesn't match the common case

**JavaScript/TypeScript Success**: Template literals have become the preferred string syntax:
```javascript
const name = "Alice";
const message = `Hello, ${name}!`;
```

This success demonstrates that making interpolation easy and default improves developer experience.

**Dada's Opportunity**: As a new language, Dada can learn from this evolution and make the convenient choice the default choice. Rather than requiring special syntax or function calls for the common case of building strings with dynamic content, Dada should support interpolation in the standard string literal syntax.

## Guide-level explanation

In Dada, string literals enclosed in double quotes support embedding expressions using curly braces:

```dada
name := "Alice"
age := 30
message := "Hello, {name}! You are {age} years old."
```

This is the default and only form of string literal in Dada. Any valid expression can be placed inside `{}`:

```dada
# Field access
greeting := "Welcome, {user.name}!"

# Method calls  
result := "The sum is {calculate_sum(a, b)}"

# Complex expressions
status := "Processing {completed}/{total} items ({(completed * 100 / total).round()}%)"
```

For cases where literal braces are needed, they can be escaped with a backslash:
```dada
json := "\{ \"name\": \"{name}\" \}"  # Produces: { "name": "Alice" }
```

### Triple-quoted strings

String literals can also be delimited by triple quotes (`"""`) to allow embedded quotes without escaping:

```dada
# No need to escape quotes
message := """She said "Hello, {name}!" with enthusiasm."""
assert message == "She said \"Hello, Alice!\" with enthusiasm."

# Triple quotes behave identically to single quotes
simple := """foo bar"""
assert simple == "foo bar"

# Interpolation works the same way
dialogue := """
    "{character1}" asked, "How are you?"
    "{character2}" replied, "I'm doing great!"
"""
```

Triple-quoted strings follow all the same rules as regular string literals, including interpolation and multiline dedenting behavior.

### Multiline String Literals

Dada supports multiline string literals with automatic indentation handling. When a string literal:
1. Begins with a newline immediately after the opening quote
2. Has each subsequent line either empty or with a consistent whitespace prefix

Then the string literal's value will be:
- The leading and trailing whitespace trimmed
- The common whitespace prefix removed from the start of each line

```dada
# Automatic indentation handling
name := "Alice"
message := "
    Hello, {name}!
    Welcome to Dada.
    This is a multiline string.
"
assert message == "Hello, Alice!\nWelcome to Dada.\nThis is a multiline string."

# Nested indentation preserved
user_name := "Bob"
login_time := "10:30 AM"
post_count := 5
report := "
    Status Report
    =============
    User: {user_name}
    
    Recent activities:
      - Logged in at {login_time}
      - Updated profile
      - Posted {post_count} messages
"
assert report == "Status Report\n=============\nUser: Bob\n\nRecent activities:\n  - Logged in at 10:30 AM\n  - Updated profile\n  - Posted 5 messages"
```

To disable automatic indentation handling, begin the string with `"\` followed by a newline. This preserves the string exactly as written, including the leading newline and all indentation:

```dada
# Preserve exact formatting (note: leading newline is preserved)
name := "Alice"
raw_message := "\
    Hello, {name}!
    Welcome to Dada.
    This is preserved exactly as written.
"
assert raw_message == "\n    Hello, Alice!\n    Welcome to Dada.\n    This is preserved exactly as written.\n"
```

To include a trailing newline in the dedented string, end with `\n`:

```dada
# With trailing newline
with_newline := "
    Line 1
    Line 2
    Line 3\n
"
assert with_newline == "Line 1\nLine 2\nLine 3\n"

# Without trailing newline (default)
without_newline := "
    Line 1
    Line 2
    Line 3
"
assert without_newline == "Line 1\nLine 2\nLine 3"
```

Interpolation works seamlessly with multiline strings:

```dada
host := "localhost"
port := 8080
db_url := "postgres://localhost/mydb"
pool_size := 10
config := "
    [server]
    host = {host}
    port = {port}
    
    [database]
    url = {db_url}
    pool_size = {pool_size}
"
assert config == "[server]\nhost = localhost\nport = 8080\n\n[database]\nurl = postgres://localhost/mydb\npool_size = 10"
```

## Reference-level explanation

### Syntax

String literals are delimited by either:
- Double quotes (`"`) 
- Triple quotes (`"""`)

Both forms may contain interpolation expressions within curly braces (`{expression}`). Triple-quoted strings allow embedded double quotes without escaping.

### Lexical Analysis

String literals with interpolation are recognized by the lexer, which understands the structure of interpolated expressions. This means that:

- Characters inside `{}` are treated as part of the interpolated expression, not the string literal
- Quotes inside interpolated expressions do not terminate the string literal

For example:
```dada
greeting := "Hello{", world"}"  # Results in: Hello, world
message := "Say {"hello"}"      # Results in: Say hello
```

The lexer tracks brace nesting to correctly identify where interpolated expressions end:
```dada
nested := "Result: {if true { "yes" } else { "no" }}"  # Results in: Result: yes
```

### Expression Evaluation

- Expressions inside `{}` are evaluated at runtime in the current scope
- Results are converted to strings following Dada's standard conversion rules
- Evaluation proceeds left-to-right
- The permission system applies normally to interpolated expressions

### Escape Sequences

- `\{` produces a literal `{`
- `\}` produces a literal `}`
- `\"` produces a literal quote (not needed in triple-quoted strings)
- `\n`, `\r`, `\t`, `\\` follow standard conventions
- Triple-quoted strings cannot contain three consecutive quote characters

### Type Requirements

- String literals have type `my String`
- Interpolated expressions must produce values that can be converted to strings
- This is checked at compile time
- The exact conversion mechanism depends on Dada's trait/interface system (future RFC)

## Frequently asked questions

**Q: Why make interpolation the default instead of having separate syntax like backticks?**
A: Experience from Rust and other languages shows that building strings with dynamic content is the common case. Making the common case require special syntax (format macros, template literals, etc.) creates friction. Dada chooses to optimize for the common case.

**Q: What about purely static strings with no interpolation?**
A: The compiler can easily detect string literals that contain no interpolation expressions and optimize them accordingly.

**Q: Why `{}` instead of `${}` like JavaScript?**
A: The simpler `{}` syntax is more consistent with Rust's format strings and requires less visual noise. Since interpolation is the default, the syntax should be as lightweight as possible.

**Q: Why `\{` instead of `{{` to escape braces?**
A: Two reasons. First, Dada string literals already use backslash escapes (`\n`, `\t`, `\\`, `\"`), so `\{` is consistent with the existing escape system — it would be odd to have two different escaping mechanisms in the same literal. Second, keeping `{{` free means it works as an interpolated block expression, which is useful for embedding multiline code:
```dada
result := "the value is {{
    x := foo()
    bar(x)
}}"
```
Languages like Rust and Python use `{{` for brace escaping because their interpolation lives in format macros or f-strings where backslash escapes aren't available. Dada strings have backslash escapes natively, so there's no reason not to use them.

## Future possibilities

- **Raw string literals** - A syntax to disable escape sequence processing (e.g., `r"C:\path\to\file"` would not interpret `\p`, `\t`, `\f` as escape sequences)
- **Method-based formatting** - Rather than format specifiers like `{x:02}`, Dada will use method calls like `{x.padded(2)}` to maintain syntactic consistency
- **Display trait** - Once Dada's trait system is designed, add a trait to allow interpolating expressions that don't directly produce `String` values (similar to Rust's `Display`)