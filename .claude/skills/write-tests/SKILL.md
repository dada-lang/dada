---
name: write-tests
description: Write spec-aligned Dada tests. Use when creating new test files, organizing tests to match the specification, or adding test coverage for language features.
---

# Writing Spec-Aligned Dada Tests

## Directory Structure

Tests mirror the spec directory structure under `tests/`:

```
spec/src/syntax/string-literals.md     →  tests/syntax/string_literals/
spec/src/syntax/literals.md            →  tests/syntax/literals/
```

Within each test directory, organize by spec section:

```
tests/syntax/string_literals/
├── delimiters/
│   └── quoted.dada                    # #:spec syntax.string-literals.delimiters.quoted
├── type.dada                          # #:spec syntax.string-literals.type
├── escape_sequences/
│   ├── backslash.dada                 # #:spec syntax.string-literals.escape-sequences.backslash
│   ├── invalid.dada                   # #:spec syntax.string-literals.escape-sequences.invalid
│   └── ...
└── interpolation/
    └── brace_escaping.dada            # #:spec syntax.string-literals.interpolation.brace-escaping
```

Ad-hoc tests that don't correspond to a spec paragraph go in `tests/adhoc/`.

## Spec Paragraph ID Resolution

IDs are built from the spec file path and headings:

1. **File prefix**: `spec/src/syntax/string-literals.md` → `syntax.string-literals`
   - `README.md` is special: only the parent directory becomes the prefix
2. **Heading segments**: H2+ headings, lowercased, spaces/underscores → hyphens
   - H1 is skipped (it's the page title, already in the file prefix)
3. **Block local name**: From `:::{spec} local_name` directive
   - If the first token looks like a tag (`rfc0001`, `unimpl`), there's no local name
4. **Inline sub-paragraph**: From `` {spec}`name` `` inside a block

### Examples

```markdown
# String Literals                         ← H1: skipped (in file prefix)
## Escape Sequences                       ← H2: "escape-sequences"
:::{spec} rfc0001                         ← Block: no local name (rfc0001 is a tag)
* {spec}`backslash` `\\` produces...      ← Inline: "backslash"
:::
:::{spec} invalid rfc0001                 ← Block: local name = "invalid"
:::
```

Resulting IDs:
- `syntax.string-literals.escape-sequences` (the block)
- `syntax.string-literals.escape-sequences.backslash` (inline)
- `syntax.string-literals.escape-sequences.invalid` (named block)

## Writing Test Files

### Basic test structure
```dada
#:spec syntax.string-literals.delimiters.quoted
#:skip_codegen

async fn main() {
    print("hello").await
    print("").await
}
```

### Error test (diagnostic expectations)
```dada
#:spec syntax.string-literals.escape-sequences.invalid
#:skip_codegen

async fn main() {
    print("\a").await
    #!      ^ /invalid escape
}
```

The `^` must be at the exact column of the error span on the previous line. Use `/pattern` (NO closing `/`) for regex matching.

### Type probe test
```dada
#:spec syntax.string-literals.type
#:skip_codegen

fn main() {
    let x = "hello"
    #?  ^ VariableType: String
}
```

`VariableType` shows the declared type without permissions. Use `ExprType` for expression types. Multiple `^^` carets match multi-byte spans.

## Known Issues

### Brace escaping in strings
The tokenizer's `delimited()` function doesn't skip over string literal contents when scanning for matching braces. Unbalanced `{` or `}` inside strings will confuse brace-depth tracking. **Workaround**: Use balanced `\{...\}` pairs in test strings:

```dada
# Good — balanced braces
print("\{\}").await
print("hello\{world\}").await

# Bad — unbalanced brace causes parse errors
print("\{").await
```

This will be fixed when string interpolation is implemented.

## Checklist for New Tests

1. Identify the spec paragraph ID for the feature being tested
2. Create the test file in the matching directory structure
3. Add `#:spec <paragraph-id>` annotation
4. Add `#:skip_codegen` if the test doesn't need WebAssembly generation
5. Write test code exercising the feature
6. Add `#!` annotations for expected errors or `#?` probes for type checking
7. Run with `cargo dada test --porcelain <test-file>` to verify
8. Check `.test-report.md` if the test fails
