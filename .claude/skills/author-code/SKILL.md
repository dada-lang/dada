---
name: author-code
description: Conventions for authoring Rust code in the Dada compiler. Use when writing or modifying Rust code, adding functions, or making implementation changes.
---

# Authoring Code in Dada

## Code Style

- **Match local conventions** — mimic the style of the file you're editing
- **Use existing utilities** — check for helper functions before writing new ones
- **Check dependencies first** — never assume a library is available; verify in Cargo.toml
- **Follow existing patterns** — look at neighboring code and match its style

## Documentation

### When to document
- **Complex functions** encoding Dada's semantics need thorough documentation
- **Self-evident code** — simple utility functions don't need extensive comments

### Documentation pattern
- Use doc comments (`///`) with high-level explanation
- Include concrete Dada code examples showing the feature being implemented
- Use inline comments referencing back to the examples

```rust
/// Type checks a method call expression like `obj.method(args)`.
///
/// # Example
/// ```dada
/// let p = Point(x: 10, y: 20)
/// p.distance(other)  # <-- we are type checking this
/// ```
///
/// The method resolution follows these steps:
/// 1. Determine the type of the receiver (`p`)
/// 2. Look up the method in the type's namespace
/// 3. Check argument compatibility
fn type_check_method_call(...) {
    // Step 1: Get receiver type (Point in our example)
    let receiver_ty = self.type_of(receiver);

    // Step 2: Resolve method - this handles the lookup of `distance`
    // in the Point type's method table
    let method = self.resolve_method(receiver_ty, method_name)?;
}
```

## Insight Comments (`💡`)

Use `💡` comments to capture non-obvious constraints and reasoning for future sessions.

### Format
- **Preamble comment** on functions: explain the overall algorithmic or architectural choice
- **Inline comments** at the start of logical blocks: explain reasoning for that block
- **Before modifying code with `💡` comments**: pause and consider whether the reasoning affects your planned changes

### Decision boundaries

Annotate non-obvious decisions — skip self-explanatory code:
- ❌ `// 💡: Using a loop to iterate through items`
- ✅ `// 💡: Using manual iteration instead of map() to handle partial failures gracefully`

Document constraint-driven choices:
- ❌ `// 💡: Using async/await for the API call`
- ✅ `// 💡: Using async/await because this API has 2-second response times that would block the UI`

Document tradeoffs and alternatives:
- ✅ `// 💡: Using Redis instead of in-memory cache because we need persistence across server restarts`

Capture consistency requirements:
- ✅ `// 💡: Using Result<T, E> pattern to match error handling in auth.rs and database.rs modules`

### Guidelines
1. **Focus on decisions with alternatives** — if there was only one way to do it, don't annotate
2. **Update annotations when modifying code** — ensure reasoning still matches implementation
3. **Be concise but specific** — future sessions should understand the decision quickly

## Error Handling

- Use diagnostics infrastructure from `dada-ir-ast`
- Provide helpful error messages with source spans
- Follow existing error formatting patterns

## References

- [Architecture](.development/architecture.md) — compiler structure and design
- [Patterns](.development/patterns.md) — full code conventions
- [Documentation](.development/documentation.md) — rustdoc guidelines
