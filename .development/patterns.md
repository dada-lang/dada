# Code Patterns and Conventions

This document describes established patterns in the Dada codebase. When contributing, follow these conventions for consistency.

## General Principles

- **Follow existing patterns** - Look at neighboring code and match its style
- **Check dependencies first** - Never assume a library is available; verify in Cargo.toml
- **Security first** - Never expose or log secrets; never commit keys

## Code Style

- **Match local conventions** - Mimic the style of the file you're editing
- **Use existing utilities** - Check for helper functions before writing new ones

## Documentation Style

### When to Document
- **Complex functions** - Functions encoding Dada's semantics need thorough documentation
- **Self-evident code** - Simple utility functions don't need extensive comments

### Documentation Pattern
- **Function-level docs** - Use doc comments (`///`) to explain high-level functionality
- **Concrete examples** - Include Dada code examples showing the feature being implemented
- **Implementation comments** - Use inline comments to explain specific parts, referencing back to the examples

Example:
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

## Testing Patterns

- Test files go in `tests/` with `.dada` extension
- Use `#:skip_codegen` for tests that don't need WebAssembly generation
- Parser tests in `tests/parser/`
- Type checking tests in `tests/type_check/`
- Experimental features in `tests/spikes/`

## Error Handling

- Use diagnostics infrastructure from `dada-ir-ast`
- Provide helpful error messages with source spans
- Follow existing error formatting patterns

## Documentation

See [documentation.md](./documentation.md) for rustdoc guidelines.