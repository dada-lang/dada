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

Implementation details are documented in rustdoc within each crate. Key crates with comprehensive docs:

- **`dada-lang`** — High-level language and compiler overview (`cargo doc --open`)
- **`dada-parser`** — Parser architecture, `Parse` trait, commitment model
- **`dada-ir-sym`** — Symbolic IR, type system, permissions
- **`dada-check`** — Type checking orchestration
