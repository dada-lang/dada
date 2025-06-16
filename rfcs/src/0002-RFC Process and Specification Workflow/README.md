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

Example spec paragraph evolution:
```markdown
r[syntax.string-literals.basic]
String literals are enclosed in double quotes: `"hello"`.
```

After RFC-123 implementation begins:
```markdown
r[syntax.string-literals.basic]
rfc[123]
String literals support both single and double quotes: `"hello"` or `'hello'`.
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

### Specification Paragraph Annotation System

**Paragraph identifiers**: Each spec paragraph has a semantic ID using `r[topic.subtopic.detail]` syntax.

**RFC annotations**: RFC-modified paragraphs include `rfc[123]` or `rfc[123, 456]` on the line following the paragraph ID.

**Version management rules**:
- Paragraphs with RFC annotations can be freely modified without version bumps
- Removing RFC annotations requires creating a new version (e.g., `basic` → `basic.1`) 
- Multiple RFCs can modify the same paragraph using comma-separated syntax
- Deleted features use `rfc[123]` followed by "Deleted." text

### Test Validation System

**Syntax**: Tests use `#:spec topic.subtopic.detail` in file headers to declare which spec paragraphs they validate.

**Prefix matching**: Test references match all sub-paragraphs (e.g., `#:spec syntax.string-literals` matches `syntax.string-literals.basic`, `syntax.string-literals.escape-sequences`, etc.).

**Validation**: The test runner parses the specification mdbook to extract all `r[...]` labels and validates that `#:spec` references point to existing paragraphs.

### Tooling Implementation Options

Two primary approaches for implementing the interactive specification viewer:

#### Option 1: Enhanced mdbook with Custom Preprocessor
- **Current approach**: Extend existing mdbook preprocessor
- **Interactive filtering**: JavaScript-based show/hide with CSS classes
- **Benefits**: Single build, dynamic user control, familiar toolchain
- **Limitations**: All content processed, client-side filtering only

#### Option 2: Custom Sphinx Extension with MyST Markdown
- **Native conditional content**: Sphinx `{only}` directive for build-time filtering
- **Multiple builds**: Generate stable, RFC-specific, and combined variants
- **Benefits**: True content exclusion, mature cross-referencing, extensible
- **Limitations**: Build-time only, no dynamic user control

#### Recommended Hybrid Approach
Develop a **custom Sphinx extension** that generates **layered HTML output**:
- Process all content during build but generate interactive layers
- Default view shows stable content only
- JavaScript controls reveal RFC variants with visual indicators
- Combines the benefits of both approaches

## Frequently asked questions

### Why not use build-time filtering only?

While Sphinx's `{only}` directive provides clean content exclusion, the desired user experience requires **dynamic interaction**. Users should be able to toggle RFC content on/off while reading to understand the differences, not navigate between separate build artifacts.

### Why semantic paragraph IDs instead of numeric ones?

Semantic identifiers (e.g., `syntax.string-literals.basic` vs `4.2.1`) remain stable during specification reorganization. Numeric IDs break when sections are reordered, but semantic names describe the content regardless of document structure.

### How do multiple concurrent RFCs avoid conflicts?

The annotation system allows `rfc[123, 456]` for collaborative modifications. The "source code" model encourages early integration of spec changes during RFC development, making conflicts visible immediately rather than at merge time.

### What happens to RFC annotations after implementation?

Once an RFC is fully implemented and the feature is stable:
1. Remove the `rfc[123]` annotation from the paragraph
2. Create a versioned paragraph (e.g., `basic.1`) to maintain history
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