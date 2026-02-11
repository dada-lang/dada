# TODO and Session Notes

*This file tracks ongoing work and provides context for resuming sessions*

## Current Status
RFC drafted with multiline string support, ready for implementation planning

## Open Questions
- Exact string conversion mechanism (depends on trait/interface system)
- Raw string syntax (future RFC)
- Precise rules for determining common whitespace prefix in edge cases

## Next Steps
- Begin implementation planning
- Define lexer changes needed
- Design AST representation for interpolated strings

## Session Notes

### 2025-01-06
- Renamed RFC directory from `0001-interpolated-strings` to `0001-string-literals`
- Added multiline string literal design:
  - Automatic dedenting when string starts with newline after opening quote
  - Common whitespace prefix removal
  - `"\` syntax to disable dedenting
  - `\n` before closing quote for trailing newline
- Spec paragraphs are authored directly in `spec/src/syntax/string-literals.md` (not in a separate RFC spec.md)
- Created executable examples using `assert` syntax
- Added design tenets section with three core principles:
  - Do what I mean
  - Rust-like syntax
  - Simple escape hatch
- Added triple-quoted string literals (`"""`) for embedded quotes
- Restructured spec paragraphs with cleaner rule separation