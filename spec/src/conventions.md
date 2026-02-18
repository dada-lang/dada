# Conventions

This chapter describes the conventions used throughout this specification.

## Paragraph References

Specification paragraphs use MyST directive syntax with the `{spec}` directive:

```markdown
:::{spec} local-name rfc123
Paragraph content.
:::
```

### ID Resolution

Paragraph IDs are resolved automatically from context:

1. **File path**: `syntax/string-literals.md` contributes prefix `syntax.string-literals`
2. **Section headings**: `## Escape Sequences` contributes segment `escape-sequences`
3. **Local name**: The name in the `:::{spec}` directive (e.g., `invalid`)

These combine to form the full ID: `syntax.string-literals.escape-sequences.invalid`

The local name is optional. A directive with only tags uses the heading context as its ID:

```markdown
## Type

:::{spec} rfc0001 unimpl
String literals have type `my String`.
:::
```

This paragraph's ID is `syntax.string-literals.type` (file prefix + heading).

### Inline Sub-paragraphs

List items within a `:::{spec}` block can be marked as individually referenceable
sub-paragraphs using the `` {spec}`name` `` syntax:

```markdown
:::{spec} rfc0001 unimpl
There are multiple forms of string literals:

* {spec}`quoted` Single-quoted string literals begin with `"` and end with `"`.
* {spec}`triple-quoted` Triple-quoted string literals begin with `"""` and end with `"""`.
:::
```

Under `## Delimiters` in `syntax/string-literals.md`, this creates:
- `syntax.string-literals.delimiters` (parent paragraph)
- `syntax.string-literals.delimiters.quoted` (sub-paragraph)
- `syntax.string-literals.delimiters.triple-quoted` (sub-paragraph)

Each sub-paragraph gets its own linkable anchor in the rendered output.

### RFC and Status Annotations

Paragraphs include tags after the optional local name:

```markdown
:::{spec} local-name rfc123 unimpl
Content added by RFC 123, not yet implemented.
:::
```

Available tags:
- `rfcN` — content added or modified by RFC N
- `!rfcN` — content deleted by RFC N
- `unimpl` — specified but not yet implemented

Multiple tags can be combined: `:::{spec} local-name rfc123 rfc456 unimpl`

### Test Annotations

Tests reference spec paragraphs using `#:spec` comments with the fully-qualified ID:

```dada
#:spec syntax.string-literals.delimiters.quoted
```

These labels serve multiple purposes:
- Cross-referencing within the specification
- Linking from RFC documents
- Test validation via `#:spec` annotations in `.dada` test files

Identifiers use semantic names rather than numbers to remain stable as the specification evolves.

## EBNF Notation

This specification uses Extended Backus-Naur Form (EBNF) to describe syntax.
Standard EBNF operators apply:

- `A*` — zero or more repetitions of A
- `A+` — one or more repetitions of A
- `A?` — optional A
- `A | B` — A or B
- `` `keyword` `` — a literal terminal
- `ε` — the empty production

In addition, this specification uses the following shorthand
for comma-separated lists with optional trailing commas:

- `A,*` — zero or more comma-separated occurrences of A
- `A,+` — one or more comma-separated occurrences of A

## Normative Language

This specification uses the following terms to indicate requirements:
- **must**: An absolute requirement
- **must not**: An absolute prohibition
- **should**: A strong recommendation
- **should not**: A strong recommendation against
- **may**: An optional feature or behavior
