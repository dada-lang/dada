# RFC Workflow

## Overview
Dada uses an RFC (Request for Comments) process for proposing and tracking language changes. The workflow involves three documentation sites:

- **RFCs** (`rfcs/` → `dada-lang.org/rfcs`) - Design proposals and decisions
- **Specification** (`spec/` → `dada-lang.org/spec`) - Authoritative language specification
- **User Docs** (`book/` → `dada-lang.org`) - Tutorials and guides

## RFC Structure
Each RFC lives in its own directory under `rfcs/src/`:

```
rfcs/src/
  SUMMARY.md                    # Table of contents, organized by status
  0001-feature-name/
    README.md                   # The RFC document
    impl.md                     # Implementation progress tracking
    spec.md                     # Draft specification text
    todo.md                     # Ongoing work and session context
    examples/                   # Example code (optional)
```

## Creating an RFC
1. Choose the next sequential number (e.g., `0002` if `0001` exists)
2. Create directory `rfcs/src/XXXX-feature-name/`
3. Write RFC in `README.md` using the template below
4. Create placeholder `impl.md` and `spec.md` files
5. Update `rfcs/src/SUMMARY.md` to include your RFC in the appropriate status section

## RFC Template
```markdown
# RFC-XXXX: Title

---
status: active
tracking-issue: #123  # optional
implemented-version: 0.1.0  # when implemented
---

## Summary
Brief one-paragraph explanation

## Motivation
Why are we doing this? What use cases does it support?

## Design tenets
Ordered list of design principles that guide decisions (most important first).
Each tenet should have a short, memorable phrase and optional clarifying text.

Example:
1. **Correctness over convenience** - We never sacrifice memory safety for ergonomics
2. **Common case is concise** - Optimize syntax for frequent operations
3. **Explicit over implicit** - When safety matters, require clear intent

## Guide-level explanation
Explain the proposal as if teaching it to another Dada programmer

## Reference-level explanation
Technical details and edge cases

## Frequently asked questions
Common questions and concerns about this design

## Future possibilities
What future extensions or changes might this enable?
```

## Iterative Development
RFCs, implementation, and specification evolve together:

1. **Design phase**: Focus on RFC document, capture ideas in spec.md
2. **Implementation**: Update impl.md with progress, refine spec.md based on experience
3. **Completion**: Integrate spec.md into main specification, update RFC status

## Specification Guidelines
- Use semantic paragraph identifiers: `r[topic.subtopic.detail]`
- Each paragraph should specify one testable behavior
- Tests reference spec paragraphs: `#:spec topic.subtopic.detail`
- Examples: `r[syntax.string-literals.escape-sequences]`, `r[permissions.lease.transfer-rules]`

## Cross-referencing
- RFCs can reference future spec sections
- Specs include non-normative RFC references for context
- Tests annotate which spec paragraphs they verify
- Keep paragraph IDs stable during reorganization

## RFC Status Lifecycle
- **active**: Under discussion and design
- **accepted**: Design approved, ready for implementation
- **implemented**: Complete with working code
- **rejected**: Not proceeding (kept for historical record)
- **withdrawn**: Author chose not to proceed

## Decision Process
@nikomatsakis acts as BDFL (Benevolent Dictator For Life) for RFC acceptance decisions.

## RFC Authorship Style Guide

When writing RFCs, follow these style preferences:

### Content Principles
- **Minimal and focused** - Include only essential content. Three design tenets are better than five if they capture the core ideas.
- **Language features over implementation** - Focus on user-facing design decisions. Avoid discussing optimizations or implementation details unless central to the design.
- **Practical escape hatches** - Prefer simple, single-character solutions (like `"\`) over complex mechanisms.

### Writing Examples
- **Executable over descriptive** - Use `assert` statements that can actually run rather than comments explaining results
- **Complete examples** - Include variable definitions so examples are self-contained

### Specification Style
- **Atomic rules** - Write separate rules for each concept rather than compound rules
- **Clear precedence** - When syntax can be ambiguous (like `"""` vs empty string + quote), add explicit clarifying notes
- **Clean separation** - One rule for types, separate rules for syntax variations, so general rules don't need to enumerate all cases

### Language Precision
- **Be specific** - "leading newline is preserved" is clearer than "preserved exactly as written"
- **Active voice** - "String literals have type `my String`" not "The type of string literals is `my String`"
- **Consistent terminology** - Pick terms and use them consistently throughout