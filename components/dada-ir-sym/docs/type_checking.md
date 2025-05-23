# Type Checking Pipeline

The type checking module orchestrates the transformation of parsed AST into validated Symbolic IR. This is where Dada's type system operates.

## High-Level Process

Type checking in Dada follows a multi-phase approach:

1. **Environment Setup** - Create typing environments for scopes
2. **Symbol Resolution** - Bind names to their definitions  
3. **Type Inference** - Infer types for expressions and variables
4. **Permission Checking** - Validate ownership and borrowing
5. **Constraint Solving** - Resolve type and permission constraints
6. **Validation** - Ensure all semantic rules are satisfied

## Key Modules

### Core Infrastructure
- **[`env`](mod@env)** - Typing environments that track variables and their types
- **[`scope`]** - Lexical scoping and name resolution
- **[`runtime`]** - Runtime type information and checking context

### Type System
- **[`inference`]** - Hindley-Milner type inference with permission extensions
- **[`subtype`]** - Subtyping relations and coercion rules
- **[`predicates`]** - Permission checking predicates (`my`, `our`, `mut`, `ref`)
- **[`red`]** - Ownership analysis ("Red" type system)

### Expression and Statement Checking  
- **[`exprs`]** - Expression type checking and inference
- **[`statements`]** - Statement validation and control flow
- **[`blocks`]** - Block expressions and local scoping
- **[`places`]** - Place expressions and borrowing analysis

### Declaration Processing
- **[`functions`]** - Function signature and body checking
- **[`generics`]** - Generic type parameter validation  
- **[`signature`]** - Function signature processing

### Utilities
- **[`resolve`]** - Name resolution utilities
- **[`report`]** - Error collection and diagnostic reporting
- **[`member_lookup`]** - Method and field resolution

## The Checking Context

Most type checking operations take place within a checking context that maintains:

- **Type environment** - Maps variables to their types
- **Permission environment** - Tracks ownership states  
- **Scope information** - Current lexical scope and accessible names
- **Constraint accumulator** - Collects type and permission constraints
- **Error collector** - Accumulates diagnostic messages

## Error Handling Philosophy

Dada's type checker follows an "error recovery" approach:
- Continue checking even after encountering errors
- Collect multiple errors to report them all at once
- Use error types (`SymTy::Error`) to prevent cascading failures
- Provide diagnostic messages with source locations

This allows developers to see multiple issues at once rather than fixing them one by one.

## Integration with Salsa

All type checking is implemented as Salsa queries, providing:
- **Incremental compilation** - Only recheck what changed
- **Memoization** - Cache results of expensive operations  
- **Dependency tracking** - Automatically invalidate stale results
- **Parallel execution** - Check independent modules concurrently