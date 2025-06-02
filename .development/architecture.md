# Dada Compiler Architecture

## Overview

The Dada compiler is built as a Cargo workspace using the Salsa incremental computation framework. The compiler transforms source code through multiple intermediate representations.

## Compilation Pipeline

```text
Source Code (.dada files)
    ↓
[dada-parser] → Tokens → AST
    ↓  
[dada-ir-ast] → AST with spans and diagnostics
    ↓
[dada-ir-sym] → Symbolic IR (type-checked, high-level)
    ↓
[dada-check] → Type checking orchestration
    ↓
[dada-codegen] → WebAssembly (incomplete)
```

## Key Components

### Core Pipeline Crates

- **[`dada-parser`](https://dada-lang.org/impl/dada_parser/)** - Tokenization and parsing
- **[`dada-ir-ast`](https://dada-lang.org/impl/dada_ir_ast/)** - AST representation  
- **[`dada-ir-sym`](https://dada-lang.org/impl/dada_ir_sym/)** - Symbolic IR and type system
- **[`dada-check`](https://dada-lang.org/impl/dada_check/)** - Type checking orchestration
- **[`dada-codegen`](https://dada-lang.org/impl/dada_codegen/)** - WebAssembly generation

### Supporting Infrastructure

- **[`dada-lang`](https://dada-lang.org/impl/dada_lang/)** - CLI entry point
- **[`dada-compiler`](https://dada-lang.org/impl/dada_compiler/)** - Compilation orchestration
- **[`dada-debug`](https://dada-lang.org/impl/dada_debug/)** - Debug server
- **[`dada-lsp-server`](https://dada-lang.org/impl/dada_lsp_server/)** - Language Server Protocol
- **[`dada-util`](https://dada-lang.org/impl/dada_util/)** - Shared utilities

## Documentation Approach

Implementation details are documented in rustdoc within each crate. The documentation uses:
- Module-level docs with `//!` comments
- External markdown files included via `#![doc = include_str!("../docs/overview.md")]`
- Comprehensive explanations of algorithms and design decisions

To explore implementation details, visit the crate documentation links above or run `just doc-open` locally.