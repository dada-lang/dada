This file provides Claude-specific guidance when working with the Dada compiler repository.

# Project overview

Dada is an experimental programming language by @nikomatsakis, exploring what a Rust-like language would look like if designed to feel more like Java/JavaScript. It's async-first, uses a permission-based ownership system, and compiles to WebAssembly.

# Running Dada programs

You can run a Dada program using the `cargo dada` alias:

```bash
cargo dada run <file.dada>     # Run a Dada program
```

# Running tests

To run tests, you `cargo dada test --porcelain [tests]`. The `tests` parameter is optional and is a path to a directory or a specific test file. The command will output to a JSON structure to stdout describing test results and guiding you on how to resolve test failures. The `suggestion` field of the test provides actionable guidance on how to resolve individual test failures.

# Track ongoing tasks with github issues

@.socratic-shell/github-tracking-issues.md

# Authoring code: include insightful comments

@.socratic-shell/ai-insights.md

# Codebase documentation

The `.development` directory includes numerous development guides. Consult them when appropriate:

- [**Architecture**](.development/architecture.md) - Compiler structure and design
- [**Patterns**](.development/patterns.md) - Code conventions and established patterns  
- [**Workflows**](.development/workflows.md) - Build, test, and development processes
- [**Documentation**](.development/documentation.md) - Rustdoc guidelines and standards
- [**RFC Process**](.development/rfc.md) - RFC workflow, specification development, and authorship style guide

# RFC and Specification Workflow

When working with RFCs or specifications:
- Follow the RFC workflow documented in [.development/rfc.md](.development/rfc.md)
- Keep RFC files (README.md, impl.md, spec.md, todo.md) updated iteratively as work progresses
- Use todo.md within each RFC directory to track ongoing work and session context
- Ensure cross-references between tests, specs, and RFCs remain synchronized