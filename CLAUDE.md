This file provides Claude-specific guidance when working with the Dada compiler repository.

# Project overview

Dada is an experimental programming language by @nikomatsakis, exploring what a Rust-like language would look like if designed to feel more like Java/JavaScript. It's async-first, uses a permission-based ownership system, and compiles to WebAssembly.

# Running Dada programs

You can run a Dada program using the `cargo dada` alias:

```bash
cargo dada run <file.dada>     # Run a Dada program
```

# Skills

Use these skills (via `/skill-name`) at the right moments:

- **author-code** — When writing or modifying Rust code. Covers conventions and patterns.
- **rfc-workflow** — When implementing an RFC feature: writing spec paragraphs, removing `unimpl` tags, or updating `impl.md`. **Always update the RFC's `impl.md` when you complete implementation work.**
- **write-tests** — When creating test files. Covers spec alignment and directory conventions.
- **run-tests** — When running tests or debugging failures.
- **tracking-issues** — For non-RFC long-running work only. RFC features track progress in `impl.md`, not GitHub issues.

# Codebase documentation

The `.development` directory includes numerous development guides. Consult them when appropriate:

- [**Architecture**](.development/architecture.md) - Compiler structure and design
- [**Patterns**](.development/patterns.md) - Code conventions and established patterns
- [**Workflows**](.development/workflows.md) - Build, test, and development processes
- [**Documentation**](.development/documentation.md) - Rustdoc guidelines and standards
- [**RFC Process**](.development/rfc.md) - RFC workflow, specification development, and authorship style guide
