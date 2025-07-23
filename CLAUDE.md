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

### Testing Quick Reference  
- **Run tests**: `cargo dada test` (from repo root, not component dirs)
- **AI-friendly output**: `cargo dada test --porcelain` (structured JSON output)
- **Bless references**: `UPDATE_EXPECT=1 cargo dada test` (updates `.ref` files)

### Parsing `--porcelain` Output
JSON structure: `{summary: {total, passed, failed, duration_ms}, tests: [...]}`

For each test object:
- `status`: "pass" or "fail"  
- `reason`: failure category (only present when status is "fail")
- `annotations`: array of `#:` comments from test file
- `suggestion`: actionable guidance (or null)
- `details`: technical error information

Follow the `suggestion` field for next steps on failures.

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

## AI Memory Comments System

This codebase uses structured emoji comments to provide persistent memory for both AI and human developers:

- `❓ QUESTION:` - Open questions or unclear areas that need investigation
- `💡 ANSWER:` - Answers to previous questions or explanations of confusing code  
- `⚠️ IMPORTANT:` - Critical things not to change/break, warnings about side effects
- `🎯 PURPOSE:` - Why this code exists, design decisions, architectural reasoning

### Rules for AI
1. **NEVER remove or contradict** existing emoji comments
2. **ALWAYS preserve** these comments when editing code - copy them to new locations if code moves
3. **When user says** "We've made this mistake before, can you add an AI note to remember it?":
   - Analyze the context and type of issue
   - Choose appropriate emoji comment type
   - Add concise, specific comment addressing the root problem
   - Place strategically above the relevant code

### Comment Placement Guidelines
- **ALWAYS precede** the code being explained - never inline at end of lines
- **Above functions/classes** for high-level context and design decisions
- **Immediately before specific lines** for detailed explanations or warnings
- **Before complex logic blocks** for questions and answers about confusing code
- **Above fragile code** for important warnings about side effects

These comments travel with the code and reduce repeated explanations across development sessions.

## RFC and Specification Workflow

When working with RFCs or specifications:
- Follow the RFC workflow documented in [.development/rfc.md](.development/rfc.md)
- Keep RFC files (README.md, impl.md, spec.md, todo.md) updated iteratively as work progresses
- Use todo.md within each RFC directory to track ongoing work and session context
- Ensure cross-references between tests, specs, and RFCs remain synchronized