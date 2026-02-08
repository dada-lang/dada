# Dada Parser Overview

This document provides an architectural overview of the Dada parser, designed to help contributors (both human and AI) understand the parsing system and make informed modifications.

## Architecture

The Dada parser is a recursive descent parser with the following key characteristics:

- **Token-based**: Source text is first tokenized, then parsed from the token stream
- **Error-recovering**: The parser accumulates diagnostics and continues parsing invalid code
- **Speculative**: Can fork state and backtrack when needed
- **Deferred**: Complex nested structures (like function bodies) are parsed lazily

## Key Components

### Entry Point

The main entry point is the `parse` method from the [`SourceFileParse`](`crate::prelude::SourceFileParse`) trait, which:
1. Tokenizes the source text
2. Creates a `Parser` instance
3. Parses the module structure
4. Returns an AST and any diagnostics

## Core Abstractions

### The `Parse` Trait

All parseable constructs implement the [`Parse`](`crate::Parse`) trait. See the trait documentation for the full interface and method signatures.

### Commitment and Error Recovery

The parser follows a "commitment" model:
- If no relevant tokens are found → parsing hasn't started
- If we start consuming tokens → we're committed to parsing that construct
- If parsing fails after commitment → must report an error

This enables robust error recovery - the parser knows when it's in the middle of a construct versus when it hasn't started one yet.

### Parse vs Eat Methods

The parser distinguishes between two parsing approaches based on the commitment model:

#### `parse` methods
- **Purpose**: Attempt to parse an element, but it's OK if it's not there
- **Returns**: 
  - `Ok(Some(T))` - Successfully parsed T
  - `Ok(None)` - No instance found (no tokens consumed, no commitment)
  - `Err(ParseFail)` - Started parsing but encountered an error (tokens were consumed, committed)
- **Key principle**: Only returns `Err` after committing to parse (consuming tokens)

#### `eat` methods  
- **Purpose**: Parse an element that must be present
- **Returns**:
  - `Ok(T)` - Successfully parsed T
  - `Err(ParseFail)` - Element not found or malformed
- **Special case**: Keywords only have `eat` methods since they're either present or not

**Important**: Returning `Err` does NOT directly report an error to the user. Errors are accumulated in the parser and may be reported later depending on context.

### Token Stream

The tokenizer ([`tokenize`](`crate::tokenizer::tokenize`) in `tokenizer.rs`) produces a flat vector of tokens. Each token includes:
- Token type (keyword, identifier, operator, etc.)
- Span information
- Whether it follows a newline (for layout-sensitive parsing)

## Parsing Strategy

### Expression Parsing

Expressions use precedence climbing for binary operators:
- Operators have defined precedence levels
- The parser handles left/right associativity
- Postfix operations (field access, calls) are parsed iteratively

### Statement/Item Disambiguation

The parser uses speculative parsing to distinguish between:
- Items (functions, classes) that can appear at module level
- Statements that must be wrapped in a function

### Error Recovery

The parser employs several recovery strategies:
- Skipping to known synchronization points
- Inserting synthetic tokens for common mistakes
- Continuing with partial ASTs when possible

## Adding New Syntax

When adding new syntax to the parser:

1. **Update the tokenizer** if new tokens are needed
2. **Define the AST node** in `dada-ir-ast`
3. **Implement `Parse`** for the new construct
4. **Update parent parsers** to recognize the new syntax
5. **Add error recovery** for common mistakes

## Design Decisions

### Why Token-Based?

Pre-tokenization simplifies the parser and enables:
- Clean separation between lexical and syntactic analysis
- Efficient lookahead and backtracking
- Better error messages with token expectations

### Why Deferred Parsing?

Deferring nested content parsing:
- Improves incremental compilation performance
- Allows parallel parsing of independent functions
- Simplifies the parser state machine

### Comment Handling

Comments use `#` (not `//`) and are filtered during tokenization. This means the parser never sees comments, simplifying grammar rules.

## Common Patterns

### Parsing Lists

The [`Parse`](`crate::Parse`) trait provides helper methods for parsing multiple items, [comma-separated lists](`crate::Parse::eat_comma`), [delimited sequences](`crate::Parse::eat_delimited`), and [arbitrary separators](`crate::Parse::opt_parse_separated`). See how these are used in practice in [`generics.rs`](`crate::generics`) for type parameter lists.

### Optional Syntax

Many constructs have optional components. See [`functions.rs`](`crate::functions`) for examples of optional return types and visibility modifiers.

### Keyword-Initiated Constructs

Constructs that begin with keywords should check for the keyword before attempting to parse. See [`module_body.rs`](`crate::module_body`) for how items like `class` and `fn` are dispatched based on their leading keyword.

## Debugging Tips

- Enable trace logging to see parser decisions
- Use [`parser.fork()`](`crate::Parser::fork`) to experiment without consuming tokens
- Check `parser.cursor` to see current position
- Examine accumulated diagnostics for error context