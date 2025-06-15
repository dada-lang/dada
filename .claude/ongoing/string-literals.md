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

### Completed Tasks (2025-06-15)

#### ✅ Parser Documentation Complete
- **Tokenizer** (`tokenizer.rs`)
  - ✅ Documented `string_literal()` method with current behavior and future extensions
  - ✅ Documented token types and escape sequence handling
  - ✅ Added notes about raw text storage and missing escape interpretation
- **Parse trait** (`lib.rs`)
  - ✅ Documented commitment model and parsing methods
  - ✅ Added examples showing parse vs eat distinction  
  - ✅ Documented error handling and diagnostic accumulation
  - ✅ Documented general error recovery patterns
- **Expression parsing** (`expr.rs`)
  - ✅ Documented `base_expr_precedence()` and literal handling
  - ✅ Documented `Literal::opt_parse()` implementation
  - ✅ Added notes about current escape sequence bug

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

## Refactoring Decisions (2025-06-15)

### Literal Value Processing
**Decision**: Tokenizer should process literal values rather than storing raw text
- **Current**: Both strings and integers store raw text, interpretation happens later
- **Proposed**: Tokenizer processes escape sequences in strings, parses integers with underscores
- **Rationale**: Source spans can recover original text when needed; consistency between literal types
- **Impact**: Changes to `TokenKind::Literal`, `Literal` AST struct, and tokenizer methods
- **Status**: Deferred until after documentation phase

### ✅ FIXED: Escape Sequence Processing Bug (2025-06-15)
**Issue**: String escape sequences were validated but never interpreted
- **Solution implemented**: Added `TokenText` interned struct for processed literal content
- **Changes made**: 
  - `TokenKind::Literal` now uses `TokenText<'db>` instead of `&'input str`
  - Tokenizer processes escape sequences (`\n` → newline, `\"` → quote, etc.) when creating `TokenText`
  - Parser updated to extract processed text from `TokenText`
  - Blessed operator precedence test reference to match new AST structure
- **Status**: ✅ Complete - All tests passing, escape sequences work correctly
- **Technical approach**: Used Salsa interned structs to store processed strings while keeping tokens `Copy`