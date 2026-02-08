# Documentation Guidelines

## Rustdoc Structure

The compiler uses rustdoc for comprehensive documentation:

- **Module-level docs** - Use `//!` comments at the top of files
- **External markdown** - Include via `#![doc = include_str!("../docs/overview.md")]`
- **Cross-references** - Link between items using rustdoc syntax

### Documentation Files
Each crate can have a `docs/` directory containing:
- `overview.md` - High-level introduction to the crate
- Topic-specific files (`type_checking.md`, `permissions.md`, etc.)

These are included in the module documentation and appear in generated rustdoc.

## Link Conventions

### Cross-Crate Links
Use regular markdown links: `[text](../crate_name)`

### Intra-Crate Links  
Use rustdoc links with backticks: `[item](`path::to::item`)`

Examples:
- `[MyStruct](`crate::module::MyStruct`)`
- `[method](`Self::method_name`)`

## Code Blocks

Always specify the language:
- ` ```rust` - For Rust code
- ` ```dada` - For Dada code examples
- ` ```text` - For output, errors, or mixed content
- ` ```bash` - For shell commands

## Writing Style

- **Factual tone** - Describe what code does, not its quality
- **Concrete examples** - Use Dada code to illustrate concepts
- **Clear structure** - Organize with headers and sections
- **Avoid adjectives** - Skip words like "powerful", "elegant", "robust"

## Building Documentation

```bash
# Generate all docs
just doc

# Generate and open in browser
just doc-open

# Serve locally at http://localhost:8000
just doc-serve

# Manual generation
cargo doc --workspace --no-deps --document-private-items
```

## Documentation Coverage

Major areas that should be documented:
- Compilation pipeline and phases
- Type system and inference
- Permission system
- Error handling and recovery
- Language semantics