---
name: rfc-workflow
description: RFC and specification workflow for Dada language features. Use when working with RFCs, writing spec paragraphs, or tracking implementation progress.
---

# RFC and Specification Workflow

See [.development/rfc.md](../../.development/rfc.md) for the full RFC process. This skill focuses on the practical spec integration workflow.

## RFC Directory Structure

```
rfcs/src/NNNN-feature-name/
├── README.md     # The RFC document (design, motivation, examples)
├── impl.md       # Implementation progress tracking
└── todo.md       # Session-specific work tracking and context
```

Create new RFCs with: `cargo xtask rfc new feature-name`

## Spec Paragraph Authoring

Spec paragraphs live in `spec/src/` using MyST directive syntax.

### Block directives

```markdown
:::{spec} local-name tag1 tag2
Paragraph content describing one testable behavior.
:::
```

**Tags**:
- `rfcNNNN` — Links paragraph to an RFC (e.g., `rfc0001`)
- `unimpl` — Feature is specified but not yet implemented
- No tag for the local name means the ID comes from headings only

### Inline sub-paragraphs

Inside a block, mark sub-items with inline spec tags:

```markdown
:::{spec} rfc0001
String literals support these escape sequences:

* {spec}`backslash` `\\` produces a literal backslash.
* {spec}`newline` `\n` produces a newline.
:::
```

Each `` {spec}`name` `` creates a sub-paragraph with its own ID. Tags like `unimpl` can follow the name: `` {spec}`triple-quoted unimpl` ``.

### Paragraph ID format

```
file-prefix.heading-segment.local-name.inline-name
```

- File prefix: from path (e.g., `syntax.string-literals`)
- Heading segments: H2+ headings, lowercased, spaces → hyphens
- Local name: from `:::{spec} local-name`
- Inline name: from `` {spec}`name` ``

## Workflow: When to Put Spec Paragraphs Where

**Design is mature** → Author directly in `spec/src/` with `rfcNNNN unimpl` tags. This is the preferred approach — it validates the spec structure early.

**Design is still evolving** → Draft in the RFC's spec.md, then move to `spec/src/` during implementation.

The key insight: if you know enough to write a spec paragraph, put it in the spec. The `unimpl` tag makes it clear it's not yet implemented.

## Implementation Tracking

### impl.md

Track implementation progress in the RFC's impl.md:

```markdown
# Implementation Progress

## Status: In Progress

### Completed
- [x] Spec paragraphs drafted in spec/src/
- [x] Basic string literal parsing

### In Progress
- [ ] Triple-quoted string support

### Not Started
- [ ] String interpolation
```

### todo.md

Track session-specific context in todo.md:

```markdown
# Current Session

## Focus
What we're working on right now

## Next Steps
- Specific actionable items

## Open Questions
- Things still being figured out
```

## Cross-Referencing

- **Tests → Spec**: `#:spec syntax.string-literals.escape-sequences.backslash`
- **Spec → RFC**: `rfc0001` tag on `:::{spec}` directives
- **RFC → Spec**: Reference spec section in RFC README.md

Keep these synchronized. When adding a new spec paragraph, check if tests exist. When writing tests, add the `#:spec` annotation.

## Implementation Workflow

When implementing an RFC feature, follow this cycle for each piece of work:

1. Implement the feature in the compiler
2. Write tests with `#:spec` annotations
3. Remove `unimpl` from the spec paragraph tag
4. **Update the RFC's `impl.md`** — check off completed items, add new items discovered during implementation

Keep `impl.md` current as you go. It's the living record of what's done and what's next — don't wait until the end of a session to update it.
