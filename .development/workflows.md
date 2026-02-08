# Development Workflows

## Building and Running

### Basic Commands
```bash
# Run the Dada compiler
cargo dada run <file.dada>

# Run tests
cargo dada test

# Run a specific test file
cargo dada test tests/hello_world.dada

# Compile without running
cargo dada compile <file.dada>

# Debug mode with introspection server
cargo dada debug <file.dada>
```

### Using Just
```bash
# Run all tests
just test

# Generate and open documentation
just doc-open

# Serve documentation locally
just doc-serve
```

## Testing Workflow

1. **Write test file** - Create `.dada` file in appropriate `tests/` subdirectory
2. **Run test** - Use `cargo dada test <file>`
3. **Check output** - Tests generate `.test-report.md` files with results
4. **Use directives** - Add `#:skip_codegen` if WebAssembly generation isn't needed

## Documentation Workflow

1. **Write docs in crate** - Add to module docs or `docs/*.md` files
2. **Build locally** - Run `just doc` to verify
3. **Check links** - Ensure cross-references work
4. **View result** - Use `just doc-open` to review

## Adding New Language Features

New language features follow an RFC-driven process:

1. **Draft RFC** - Create `rfc/rfcNNNN_feature_name.md` describing user-facing behavior
2. **Supporting materials** - Add details in `rfc/rfcNNNN_feature_name/` subdirectory
3. **Discuss architecture** - Confirm major design decisions before implementation
4. **Implement feature** - Follow the RFC's agreed design
5. **Update documentation** - Document implementation in compiler rustdoc
6. **Update language spec** - Add feature to language specification when complete

The RFC should focus on motivation, user experience, and examples. Implementation details can evolve during development.

## Fixing Bugs

1. Add failing test case
2. Debug using `cargo dada debug`
3. Fix the issue
4. Verify test passes
5. Check for regressions with `just test`

## Debugging the Compiler

The debug server allows introspection of compiler internals:

```bash
cargo dada debug examples/hello.dada
# Opens web UI showing compilation steps
```