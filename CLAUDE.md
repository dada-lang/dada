This file provides Claude-specific guidance when working with the Dada compiler repository.

# Project overview

Dada is an experimental programming language by @nikomatsakis, exploring what a Rust-like language would look like if designed to feel more like Java/JavaScript. It's async-first, uses a permission-based ownership system, and compiles to WebAssembly.

# Running Dada programs

You can run a Dada program using the `cargo dada` alias:

```bash
cargo dada run <file.dada>     # Run a Dada program
```

# Codebase documentation

The `.development` directory includes numerous development guides. Consult them when appropriate:

- [**Architecture**](.development/architecture.md) - Compiler structure and design
- [**Patterns**](.development/patterns.md) - Code conventions and established patterns  
- [**Workflows**](.development/workflows.md) - Build, test, and development processes
- [**Documentation**](.development/documentation.md) - Rustdoc guidelines and standards
- [**RFC Process**](.development/rfc.md) - RFC workflow, specification development, and authorship style guide
