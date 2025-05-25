# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dada is an experimental programming language that explores what a Rust-like language would look like if designed to feel more like Java/JavaScript rather than C++. It's async-first, uses a permission-based ownership system, and compiles to WebAssembly.

## Development Commands

### Basic Commands
- `cargo run -- compile <file.dada>` - Compile a Dada source file
- `cargo run -- run <file.dada>` - Compile and execute a Dada program  
- `cargo run -- test [files]` - Run test suite on specific Dada files
- `cargo run -- debug <file.dada>` - Compile with debug server for introspection

### Testing
- `just test` - Run all tests across the workspace (equivalent to `cargo test --all --workspace --all-targets`)
- `cargo test` - Run Rust unit tests
- Test files are in `tests/` directory with `.dada` extension

### Build Tools
- `cargo xtask build` - Custom build tasks
- `cargo xtask deploy` - Deployment automation

## Architecture

The compiler is built as a Cargo workspace with these key components:

### Core Pipeline (in compilation order)
1. **`dada-parser`** - Lexing/parsing source to AST
2. **`dada-ir-ast`** - AST representation and diagnostics  
3. **`dada-ir-sym`** - Symbolic IR (type-checked, high-level representation)
4. **`dada-check`** - Type checking and semantic analysis
5. **`dada-codegen`** - WebAssembly code generation (currently incomplete)

### Supporting Components
- **`dada-lang`** - Main CLI entry point
- **`dada-compiler`** - Compilation orchestration and VFS
- **`dada-debug`** - Debug server for compiler introspection
- **`dada-lsp-server`** - Language Server Protocol implementation
- **`dada-util`** - Shared utilities (arena allocation, logging, etc.)

### Key Design Patterns
- **Salsa-based**: Uses incremental, memoized computation framework
- **Database pattern**: Central `Db` trait for accessing compiler state
- **Async architecture**: Built around async/await throughout

## Current Status & Constraints

- **Early development**: Core language features implemented but not production-ready
- **Codegen limitations**: Most test files have `#:skip_codegen` as WASM generation is incomplete
- **Active experimentation**: Language design still evolving (see `tests/spikes/` for experimental features)

## Language Characteristics

- **Async-first**: Functions are async by default
- **Permission system**: Uses ownership annotations (`my`, `our`, `mut`) for memory management
- **Classes and structs**: Both reference types (classes) and value types (structs)
- **Rust-inspired**: Similar memory safety guarantees with more accessible syntax
- **Comments**: Use `#` not `//`

## Test File Structure

- `tests/parser/` - Parser tests
- `tests/symbols/` - Symbol resolution tests  
- `tests/type_check/` - Type checking tests
- `tests/spikes/` - Experimental language features
- `tests/default_perms/` - Default permission inference tests

Test files use `.dada` extension and often include `#:skip_codegen` directives.

## Documentation

The compiler uses rustdoc for comprehensive documentation. Major documentation files:

### Generation Commands
- `just doc` - Generate docs for all crates (recommended)
- `just doc-open` - Generate and open docs in browser
- `just doc-serve` - Generate docs and serve locally at http://localhost:8000
- `cargo doc --workspace --no-deps --document-private-items` - Manual command equivalent to `just doc`

### Documentation Structure
- **`dada-lang`** - Main landing page and compiler overview
- **`dada-ir-sym`** - Core type system and symbolic IR documentation
- **`dada-check`** - Type checking orchestration
- **Individual modules** - Detailed documentation embedded in source

### Documentation Files
- `components/*/docs/*.md` - Extended documentation included via `include_str!`
- Inline module docs using `//!` comments
- Cross-references using `[`item`]` syntax for automatic linking

Major documentation sections:
- **Type Checking Pipeline** - Overview of the checking process
- **Permission System** - Detailed guide to Dada's ownership model  
- **Type Inference** - How Hindley-Milner inference works in Dada
- **Subtyping** - Type relationships and conversions

### Documentation Guidelines

#### Cross-Crate Links
- Use `[text](../crate_name)` format for linking to sibling crates (regular markdown links)
- Avoid bare `[crate_name]` links that rely on implicit resolution

#### Intra-Crate Links  
- Use `[item](`path::to::item`)` format with backticks around the path (rustdoc links)
- For private items, use `pub(crate)` visibility when the item needs to be documented
- Prefer concrete method names over non-existent placeholder methods
- Examples: `[MyStruct](`crate::module::MyStruct`)`, `[method](`Self::method_name`)`

#### Code Blocks
- Always specify language: ```rust, ```text, ```bash, etc.
- Use ```text for error messages, command output, or mixed syntax
- Use ```rust only for valid Rust code
- Avoid bare ``` without language specification

#### Link Style
- **Intra-crate**: `[item](`path::to::item`)` (with backticks for rustdoc resolution)
- **Cross-crate**: `[crate](../crate_name)` (without backticks for markdown links)
- Keep link text descriptive but concise

#### Writing Style
- Use factual, objective tone
- Avoid subjective adjectives like "powerful", "innovative", "elegant", "robust"
- Focus on describing what the code does, not evaluating its quality
- Let readers draw their own conclusions about the design
- Prefer "implements X" over "provides powerful X functionality"

#### Error Prevention
- Verify referenced types/methods actually exist before documenting them
- Use `--document-private-items` compatible linking for internal docs
- Test documentation builds regularly with `just doc`

## Interaction Style

- Avoid unnecessary adjectives or commentary
- Identify potential errors in reasoning and suggest fixes
- Defer to user judgment after providing analysis
- Focus on direct, factual communication

## Ongoing Work Tracking

The `.ongoing/` directory contains documentation for work in progress that may span multiple sessions:
- Each ongoing task gets its own markdown file
- Files should include: status, context, work completed, next steps
- Update files when resuming or pausing work
- Remove files when work is complete