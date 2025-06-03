# CLAUDE.md

**IMPORTANT: Always check this file and `.development/` FIRST before analyzing the codebase or answering questions about established patterns.**

This file provides Claude-specific guidance when working with the Dada compiler repository.

## Behavior Guidelines

1. **Check patterns first** - Before exploring or rediscovering, consult:
   - This file for Claude-specific instructions
   - `.development/` directory for established patterns and documentation

2. **Track ongoing tasks** - always update the appropriate file in `.claude/ongoing` with status reports after reaching a milestone or finishing a major task. If we are beginning a new task, offer to create a directory in `.claude/ongoing` to track it.

3. **Follow established patterns** - Use the conventions documented in `.development` rather than inferring new ones

## Project Documentation Structure

**For Contributors:** See [CONTRIBUTING.md](CONTRIBUTING.md)

**Detailed Development Guides:**
- [**Architecture**](.development/architecture.md) - Compiler structure and design
- [**Patterns**](.development/patterns.md) - Code conventions and established patterns  
- [**Workflows**](.development/workflows.md) - Build, test, and development processes
- [**Documentation**](.development/documentation.md) - Rustdoc guidelines and standards
- [**RFC Process**](.development/rfc.md) - RFC workflow, specification development, and authorship style guide

## Quick Reference

### Project Overview
Dada is an experimental programming language by @nikomatsakis, exploring what a Rust-like language would look like if designed to feel more like Java/JavaScript. It's async-first, uses a permission-based ownership system, and compiles to WebAssembly.

### Essential Commands
```bash
cargo dada run <file.dada>     # Run a Dada program
cargo dada test               # Run test suite  
just test                     # Run all tests
just doc-open                 # Generate and open documentation
```

### Language Characteristics
- **Async-first**: Functions are async by default
- **Permission system**: Uses `my`, `our`, `leased` annotations for memory management
- **Two type kinds**: Classes (reference types) and structs (value types)
- **Comments**: Use `#` not `//`

## Claude-Specific Instructions

### When Asked About Established Patterns
Always check the relevant file in `.development/` before doing exploration or analysis.

### When Beginning Tasks
1. **Discuss the plan** - Talk through the approach and get alignment on the strategy
2. **Wait for explicit go-ahead** - Don't start making edits until explicitly asked to begin implementation
3. **Confirm scope** - Ensure we agree on what will be changed before proceeding

### Working with RFCs and Specifications
Before suggesting edits to RFCs, specifications, or code:
1. **Present your understanding** - Explain what changes you think are needed and why
2. **Propose the approach** - Outline the specific edits you would make
3. **Wait for agreement** - Only proceed with edits after explicit approval
4. **Iterative refinement** - Make changes incrementally, allowing for course corrections

This is especially important for design documents where the exact phrasing and structure matter significantly.

### When Discovering New Patterns
Update the appropriate documentation files to capture the knowledge for future sessions.

### Documentation Preferences
When creating or updating documentation:
- **Design for AI understanding** - Write docs that help future AI sessions navigate the codebase
- **Use rustdoc links** - Link all Rust items with backticks: `[`item`](`crate::path::to::item`)`  
- **Reference real code** - Point to actual implementations rather than embedding examples
- **Concepts before details** - Introduce key concepts early to frame subsequent explanations
- **Document private items** - Internal documentation is valuable; use `just doc` for proper builds
- **Precision matters** - Be exact about semantics (e.g., "Err doesn't report errors directly")

### Multi-Session Work Tracking
Use `.claude/ongoing/` for work that spans multiple sessions. Include status, context, work completed, and next steps.

## RFC and Specification Workflow

When working with RFCs or specifications:
- Follow the RFC workflow documented in [.development/rfc.md](.development/rfc.md)
- Keep RFC files (README.md, impl.md, spec.md, todo.md) updated iteratively as work progresses
- Use todo.md within each RFC directory to track ongoing work and session context
- Ensure cross-references between tests, specs, and RFCs remain synchronized