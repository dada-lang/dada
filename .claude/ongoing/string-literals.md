# Ongoing: Support Interpolated String Literals

## Overall Task
Implement interpolated string literals in Dada, including both single (`"`) and triple (`"""`) quoted variants with automatic dedenting for multiline strings.

## Progress Summary

### Completed (2025-01-06)
- ✅ Renamed RFC directory from `0001-interpolated-strings` to `0001-string-literals`
- ✅ Drafted RFC-0001 with:
  - Design tenets (Do what I mean, Rust-like syntax, Simple escape hatch)
  - Single and triple-quoted string literals
  - Multiline string support with automatic dedenting
  - `"\` escape hatch for raw multiline strings
  - Executable examples using `assert` syntax
- ✅ Created parser overview documentation (`dada-parser/docs/overview.md`)
  - Included in lib.rs via rustdoc
  - Documented parse vs eat distinction
  - Added commitment model explanation
  - Linked to actual code examples

### Next Session Tasks

#### 1. Continue Parser Documentation
Focus areas for documentation:
- **Tokenizer** (`tokenizer.rs`)
  - How string literals are tokenized
  - Token types and structure
  - Handling of quotes and escape sequences
- **Parse trait** (`lib.rs`)
  - Full trait documentation with examples
  - Helper methods and their use cases
  - Error handling patterns

Suggested additional documentation targets:
- **Expression parsing** (`expr.rs`) - Since strings are expressions
- **Literal parsing** - How literals are currently handled
- **Error recovery** - Important for string literal edge cases

#### 2. Implement Triple-Quoted Strings
- Update tokenizer to recognize `"""` delimiters
- Ensure `"""` is not parsed as empty string + quote
- Handle embedded quotes without escaping
- Maintain all existing string literal features

#### 3. Implement String Interpolation
- Lexer changes to recognize `{}` within strings
- Track brace nesting for complex expressions
- Parser changes for interpolated expressions
- AST representation for interpolated strings
- Type checking for interpolation expressions

## Key Design Decisions
- Interpolation is default behavior (no special syntax needed)
- `{}` for interpolation (Rust-like, not `${}`)
- Multiline strings auto-dedent when starting with newline
- `"\` prefix disables all "magic" (dedenting, eventually interpolation)

## Implementation Order Rationale
1. Documentation first - Understand existing patterns
2. Triple quotes - Simpler change, good warm-up
3. Interpolation - Most complex, builds on understanding

## Context for Next Session
- Use `just doc` to build documentation with private items
- Follow established parsing patterns (check existing literal parsing)
- Consider error recovery for malformed string literals
- Remember parse methods return `Ok(None)` if no commitment