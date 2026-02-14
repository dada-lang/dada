# Implementation notes

*This file tracks implementation progress for RFC-0001: String Literals*

## Status
In progress

## Completed
- [x] Escape sequence processing (`\n`, `\t`, `\\`, `\"`, `\{`, `\}`, `\r`)
- [x] Triple-quoted strings (disambiguation, termination, embedded quotes)
- [x] String type (`my String`)
- [x] Invalid escape sequence errors
- [x] Brace escaping (`\{`, `\}`)
- [x] Multiline strings: leading newline removal, trailing whitespace removal, auto-dedenting
- [x] Escape sequences treated as content during dedenting
- [x] Raw strings (`"\` prefix disables dedenting)
- [x] Ast probe infrastructure for tokenizer-level TDD

## Remaining
- [ ] String interpolation: curly brace expressions inside strings
- [ ] Lexer brace nesting depth tracking
- [ ] Nested quotes inside interpolated expressions
- [ ] Interpolation scope — evaluated in enclosing scope
- [ ] Interpolation evaluation order — left-to-right
- [ ] Type checking for interpolated expressions
- [ ] Permission system for interpolated expressions
- [ ] String conversion mechanism — blocked on trait/interface RFC

## Spec Paragraphs
14/22 spec paragraphs implemented in `spec/src/syntax/string-literals.md`.
8 remaining: 7 interpolation + 1 string conversion.

## Notes
- Spec paragraphs authored directly in the spec (not in `rfcs/src/0001-string-literals/spec.md`),
  validating the RFC-0002 workflow
- Ast probe (`#? Ast:`) enables TDD for tokenizer-level features
- `process_escape_sequences()` standalone function duplicates logic from `Tokenizer::escape_sequence()` —
  if escape rules change, both must be updated
