# RFC-0002: RFC Process and Specification Workflow

---
status: draft
---

## Summary

Establish a comprehensive RFC and specification workflow that enables incremental development of language features while maintaining clear separation between design documentation (RFCs) and normative specification text.

## Design tenets

1. **Incremental integration** - RFCs integrate spec text during implementation, not after acceptance
2. **Visual discoverability** - Users can see stable content by default with clear indicators of pending changes
3. **Flexible tooling** - Choose tools that support complex conditional content and interactive documentation

## Motivation

Dada needs a systematic way to:
- Document language design decisions and rationale (RFCs)  
- Maintain authoritative specification text that evolves with the language
- Link tests to specific specification paragraphs for validation
- Enable developers to see how RFCs would change the current specification

Currently, we have basic RFC infrastructure but lack the integration between RFCs, specifications, and tests needed for effective language development.

## Guide-level explanation

The workflow supports concurrent RFC development while maintaining spec stability:

### RFC-to-Spec Integration

When an RFC reaches active implementation:

1. **Spec text is written** directly in the main specification with RFC annotations
2. **Tests reference spec paragraphs** using `#:spec` comments for validation  
3. **Multiple views available** - stable spec vs RFC-enhanced variants
4. **Visual indicators** show where RFC content differs from stable

Example spec paragraph using MyST directive syntax:
```markdown
:::{spec} syntax.string-literals.basic
String literals are enclosed in double quotes: `"hello"`.
:::
```

After RFC-123 implementation begins, add the RFC tag:
```markdown
:::{spec} syntax.string-literals.basic rfc123
String literals support both single and double quotes: `"hello"` or `'hello'`.
:::
```

New paragraphs introduced by an RFC:
```markdown
:::{spec} syntax.string-literals.raw rfc123
Raw string literals use backticks and preserve whitespace.
:::
```

Content deleted by an RFC uses the `!` prefix:
```markdown
:::{spec} syntax.old-feature !rfc123
This feature is removed.
:::
```

### Interactive Specification View

The specification viewer provides:
- **Default**: Stable content only
- **Visual indicators**: Badges showing available RFC variants
- **Expandable sections**: Click to reveal RFC changes inline
- **Toggle controls**: Show/hide specific RFCs globally

### Test Integration

Tests link to specification paragraphs:
```dada
#:spec syntax.string-literals.basic

class TestStringLiterals {
    assert "hello" == 'hello'  # This will fail in stable spec
}
```

The `#:spec` system uses prefix matching - `syntax.string-literals` matches all sub-paragraphs for comprehensive coverage.

## Reference-level explanation

### Specification Paragraph Syntax

Specification paragraphs use MyST directive syntax with the `{spec}` directive:

```markdown
:::{spec} <paragraph-id> [rfc-tags...]
Paragraph content.
:::
```

**Paragraph identifiers**: The first argument is always a semantic ID like `syntax.string-literals.basic`. These use dotted paths that describe the content, remaining stable during document reorganization.

**RFC tags**: Optional space-separated tags following the paragraph ID:
- `rfcN` - Content added or modified by RFC N
- `!rfcN` - Content deleted by RFC N

**Examples**:

| Directive | Meaning |
|-----------|---------|
| `:::{spec} syntax.foo` | Stable paragraph |
| `:::{spec} syntax.foo rfc123` | Modified/added by RFC 123 |
| `:::{spec} syntax.foo rfc123 rfc456` | Modified by multiple RFCs |
| `:::{spec} syntax.foo !rfc123` | Deleted by RFC 123 |
| `:::{spec} syntax.foo rfc100 !rfc200` | Added by RFC 100, later deleted by RFC 200 |

**Version management rules**:
- Paragraphs with RFC tags can be freely modified without version bumps
- Removing RFC tags (stabilizing content) may warrant creating a new paragraph version (e.g., `basic` → `basic.v2`) to maintain history
- Non-normative prose between directives remains as regular markdown

### Test Validation System

**Syntax**: Tests use `#:spec topic.subtopic.detail` in file headers to declare which spec paragraphs they validate.

**Prefix matching**: Test references match all sub-paragraphs (e.g., `#:spec syntax.string-literals` matches `syntax.string-literals.basic`, `syntax.string-literals.escape-sequences`, etc.).

**Validation**: The test runner parses the specification to extract paragraph IDs from `{spec}` directives and validates that `#:spec` references point to existing paragraphs.

### Tooling Implementation

The specification uses **MyST Markdown** with **Sphinx**, enabling:

- **Native directive support**: The `{spec}` directive integrates naturally with MyST's syntax
- **Custom Sphinx extension**: Processes `{spec}` directives to generate interactive HTML
- **Layered output**: Default view shows stable content; JavaScript controls reveal RFC variants
- **Cross-referencing**: Sphinx's mature reference system links tests, RFCs, and spec paragraphs

The custom `{spec}` directive extension:
1. Parses paragraph IDs and RFC tags from directive arguments
2. Generates HTML with appropriate CSS classes for filtering
3. Builds a paragraph registry for test validation
4. Produces visual indicators (badges) for RFC-modified content

## Frequently asked questions

### Why not use build-time filtering only?

While Sphinx supports build-time content exclusion, the desired user experience requires **dynamic interaction**. Users should be able to toggle RFC content on/off while reading to understand the differences, not navigate between separate build artifacts. The `{spec}` directive generates all content with CSS classes, enabling JavaScript-based filtering at runtime.

### Why MyST directive syntax?

MyST (Markedly Structured Text) provides a standard way to extend Markdown with directives and roles, widely used with Sphinx. Using `:::{spec}` rather than a custom syntax like `r[...]`:
- Integrates with existing MyST tooling and editors
- Provides a familiar pattern for contributors who've used Sphinx/RST
- Enables a single directive to carry both paragraph ID and RFC metadata
- Allows the spec to leverage Sphinx's ecosystem (cross-references, indexing, etc.)

### Why semantic paragraph IDs instead of numeric ones?

Semantic identifiers (e.g., `syntax.string-literals.basic` vs `4.2.1`) remain stable during specification reorganization. Numeric IDs break when sections are reordered, but semantic names describe the content regardless of document structure.

### How do multiple concurrent RFCs avoid conflicts?

The directive syntax allows multiple RFC tags: `:::{spec} topic.foo rfc123 rfc456`. The "source code" model encourages early integration of spec changes during RFC development, making conflicts visible immediately rather than at merge time.

### What happens to RFC tags after implementation?

Once an RFC is fully implemented and the feature is stable:
1. Remove the `rfcN` tag from the directive (e.g., `:::{spec} syntax.foo rfc123` becomes `:::{spec} syntax.foo`)
2. Optionally create a versioned paragraph ID (e.g., `basic` → `basic.v2`) to maintain history
3. Update any tests that should only reference the new stable version

## Future possibilities

### Enhanced Test Coverage Reporting
Generate visual reports showing specification coverage, highlighting untested paragraphs and orphaned tests. This could integrate with CI to require spec coverage for new features.

### Multi-Language Specification Support
The paragraph ID system could support localized specifications by extending the annotation format to include language tags.

### RFC Impact Analysis
Tooling could analyze which tests would be affected by RFC changes, helping developers understand the scope of proposed modifications.

### Integration with Language Server
The `#:spec` references could provide jump-to-definition functionality in editors, linking test code directly to relevant specification paragraphs.