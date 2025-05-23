# Symbolic IR and Type Checking

This crate implements the core of Dada's type system - the **Symbolic IR** (intermediate representation). It takes the parsed AST from [`dada_ir_ast`] and produces a type-checked, semantically analyzed representation.

## What is Symbolic IR?

The Symbolic IR is a high-level, typed representation of Dada programs that has been:
- **Name resolved** - All identifiers are bound to their definitions
- **Type checked** - All expressions have known types
- **Permission analyzed** - Ownership annotations are validated
- **Semantically validated** - The program follows Dada's semantic rules

Unlike the AST which mirrors the source syntax, the Symbolic IR normalizes the program into a form suitable for code generation or interpretation.

## Architecture Overview

The type checking pipeline consists of several interconnected modules:

```text
AST → [symbol conversion] → Symbolic IR → [check] → Validated Symbolic IR
                               ↓
                          [inference] ← [predicates]
                               ↓           ↓  
                           [subtype] → [red types]
```

### Core Modules

- **[`crate::check`]** - Main type checking orchestration and algorithms
- **[`crate::ir`]** - Symbolic IR data structures and core types  
- **[`crate::well_known`]** - Built-in types and operations

### Type System Components

The type checker implements several analyses:

1. **Type Inference** (see [`check::inference`](crate::check::inference)) - Hindley-Milner style inference adapted for OOP
2. **Permission Analysis** (see [`check::predicates`](crate::check::predicates)) - Ownership and borrowing validation  
3. **Subtyping** (see [`check::subtype`](crate::check::subtype)) - Structural and nominal subtype relations
4. **Red Types** (see [`check::red`](crate::check::red)) - Ownership analysis for complex permission patterns

## Key Concepts

### Types in Dada

Dada has several categories of types:

- **Primitive types** - `Int`, `Float`, `String`, `Bool`
- **Class types** - Reference types with identity (like Java objects)
- **Struct types** - Value types without identity  
- **Generic types** - Parametric types with type/permission parameters
- **Future types** - For async computation results

### Permissions

Dada's ownership system uses four core permissions:

- **`my`** - Unique ownership (like Rust's owned values)
- **`our`** - Shared ownership (like Rust's `Arc<T>`)  
- **`mut`** - Mutable access (like Rust's `&mut T`)
- **`ref`** - Immutable access (like Rust's `&T`)

### Places vs Values

The type system distinguishes between:
- **Places** - Memory locations that can be read from or written to
- **Values** - The data stored in those places

This distinction enables borrowing and the permission system.

## Compilation Database

The entire type checking process is built on the [Salsa](https://github.com/salsa-rs/salsa) incremental computation framework. The main interface is the [`Db`] trait, which provides memoized access to all compiler queries.

Key query categories:
- **Symbolization queries** - Convert AST nodes to Symbolic IR
- **Type checking queries** - Validate and infer types  
- **Permission queries** - Check ownership constraints
- **Diagnostic queries** - Collect and report errors

## Error Handling

Type checking errors are accumulated rather than causing immediate failure. This allows the compiler to report multiple errors at once and continue analysis even in the presence of some errors.

See [`check::report`](crate::check::report) for the error reporting infrastructure.