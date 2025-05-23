# Type Checking Orchestration

This crate provides the high-level orchestration for Dada's type checking process. It implements the [`Check`] trait that defines what it means for a Dada program to successfully compile.

## Purpose

While the detailed type checking logic lives in [`dada_ir_sym::check`], this crate provides:
- **Top-level entry points** for type checking entire programs
- **Orchestration logic** that coordinates different phases of checking
- **The [`Check`] trait** that unifies type checking across different AST nodes

## The Check Trait

The core abstraction is the [`Check`] trait:

```rust
pub trait Check<'db> {
    fn check(&self, db: &'db dyn crate::Db);
}
```

This trait is implemented for all major AST and IR nodes:
- **[`SourceFile`]** - Check an entire source file
- **[`SymModule`]** - Check a module and all its items
- **[`SymFunction`]** - Check a function signature and body
- **[`SymAggregate`]** - Check class definitions and members

## Checking Pipeline

When you call `.check()` on a source file, it triggers a cascading validation:

1. **Module checking** - Validates module structure and use statements
2. **Item checking** - Validates each top-level item (classes, functions)  
3. **Signature checking** - Validates function signatures and generic parameters
4. **Body checking** - Validates function implementations
5. **Field checking** - Validates class field types

## Error Accumulation

The checking process accumulates errors rather than failing fast. This allows the compiler to report multiple issues at once.

## Integration with Symbolic IR

This crate serves as a bridge between the parsed AST and the detailed type checking in [`dada_ir_sym`]. It:
- Converts AST nodes to symbolic IR
- Invokes the appropriate type checking logic
- Ensures all necessary validations are performed

## Usage

Typically, you'll check an entire program like this:

```rust
use dada_check::Check;

// Check a source file
source_file.check(db);

// Or check a specific function
sym_function.check(db);
```

The actual type checking algorithms and detailed analysis are implemented in [`dada_ir_sym::check`].