# Conventions

This chapter describes the conventions used throughout this specification.

## Paragraph References

Specification paragraphs use MyST directive syntax with the `{spec}` directive:

```markdown
:::{spec} topic.subtopic.detail
Paragraph content.
:::
```

These labels serve multiple purposes:
- Cross-referencing within the specification
- Linking from RFC documents
- Test annotations via `#:spec topic.subtopic.detail`

Identifiers use semantic names rather than numbers to remain stable as the specification evolves. Examples include:
- `syntax.string-literals.escape-sequences`
- `permissions.lease.transfer-rules`
- `types.classes.field-access`

### RFC Annotations

Paragraphs modified by an RFC include RFC tags after the paragraph ID:

```markdown
:::{spec} syntax.foo rfc123
Content added or modified by RFC 123.
:::
```

Content deleted by an RFC uses the `!` prefix:

```markdown
:::{spec} syntax.old-feature !rfc123
This feature is removed.
:::
```

Multiple RFCs can be specified: `:::{spec} topic.foo rfc123 rfc456`

## Normative Language

This specification uses the following terms to indicate requirements:
- **must**: An absolute requirement
- **must not**: An absolute prohibition
- **should**: A strong recommendation
- **should not**: A strong recommendation against
- **may**: An optional feature or behavior