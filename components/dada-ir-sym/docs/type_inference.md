# Type Inference Architecture and Implementation

Dada's type inference system is based on Hindley-Milner type inference, extended to handle Dada's object-oriented features and permission system. The system uses an async-based constraint solving approach with concurrent task coordination.

## Architecture Overview

Type inference occurs during the AST → Symbolic IR conversion phase, specifically when checking function/method bodies through Salsa queries:

- **[`SymFunction::checked_body()`](`crate::prelude::CheckedBody::checked_body`)** - Main entry point for type inference
- **[`SymFunction::checked_signature()`](`crate::prelude::CheckedSignature::checked_signature`)** - Function signature checking  
- **[`SymField::checked_field_ty()`](`crate::prelude::CheckedFieldTy::checked_field_ty`)** - Field type checking

### Demand-Driven Compilation

Dada uses Salsa's incremental computation framework rather than strict compilation phases. When type inference needs a function's signature, it calls the appropriate Salsa query (e.g., `function.checked_signature(db)`), which handles memoization and dependency tracking automatically.

**Inference Scope**: Type inference is scoped to individual functions. Each function has its own inference context, and inference variables don't cross function boundaries.

## Async Constraint Solving

Type checking is implemented as an asynchronous process using Rust's `async`/`await`, but instead of I/O, the system awaits on type inference constraints becoming available.

### Runtime Execution

The [`Runtime::execute`](`crate::check::runtime::Runtime::execute`) method creates the async execution environment:

```rust
Runtime::execute(db, span, "check_function_body", &[&function], async move |runtime| {
    // Type checking logic here
    // Can spawn concurrent tasks and await on constraints
})
```

### Concurrency Model

The system uses structured concurrency in two ways:

1. **Structured concurrency** (`future::join`) - For parallel work where results are needed
2. **Background tasks** (spawned tasks) - For validation that doesn't affect main computation

Example from [`check_call_common`](`crate::check::exprs::check_call_common`):
```rust
// Check function arguments concurrently
for arg_result in futures::future::join_all((0..found_inputs).map(check_arg)).await {
    // Each argument checked in parallel
}
```

### Environment Forking

Concurrent tasks use [`Env::fork()`](`crate::check::env::Env::fork`) to create isolated execution contexts:
- **Shared state**: Inference variables and constraints are shared between parent and forked environments
- **Separate logging**: Each fork gets its own log handle for debugging task hierarchies

## Inference Variables

### Creation and Types

Inference variables are created via [`Env::fresh_inference_var()`](`crate::check::env::Env::fresh_inference_var`) with a [`SymGenericKind`](`crate::ir::types::SymGenericKind`):

- **Type variables** (`SymGenericKind::Type`) - Represent unknown types
- **Permission variables** (`SymGenericKind::Perm`) - Represent unknown permissions  
- **Place variables** - Not supported (and not planned)

**Type-Permission Relationship**: Every type variable automatically gets an associated permission variable, since Dada types are always `permission Type` pairs.

### Background Constraint Tasks

Creating an inference variable spawns background validation tasks:
- **[`relate_infer_bounds`]** - Ensures lower bound ⊆ upper bound consistency
- **[`reconcile_ty_bounds`]** - Type-specific constraint reconciliation

## Constraint System

### Red (Reduced) Types

Constraints use "red" (reduced) types - a canonical representation that factors types into:
- **Permission component** - `my`, `our`, `mut`, `ref`  
- **Core type component** - Memory layout (`Int`, `String`, `BankAccount`)

This factorization simplifies subtyping and constraint solving by allowing separate reasoning about permissions and core types.

### Constraint Coordination (Monitor Pattern)

The system uses a monitor-like pattern with async/await for constraint coordination:

1. **Monotonic bounds** - Constraints only get tighter over time, never looser
2. **Centralized updates** - [`Runtime::mutate_inference_var_data`](`crate::check::runtime::Runtime::mutate_inference_var_data`) is the only way to modify inference variables
3. **Automatic wake-up** - All tasks waiting on a variable are awakened when bounds change

### Waiting on Constraints

Tasks use standardized patterns for waiting on constraint availability:
- **[`Runtime::loop_on_inference_var`](`crate::check::runtime::Runtime::loop_on_inference_var`)** - Core monitor loop
- **[`Env::loop_on_inference_var`](`crate::check::env::combinator::Env::loop_on_inference_var`)** - Higher-level wrapper

Pattern:
```rust
env.loop_on_inference_var(var, |data| {
    // Return Some(result) when condition satisfied
    // Return None to keep waiting
    data.red_ty_bound(direction)
}).await
```

## Error Handling

Dada uses a "fail-soft" approach with two error propagation mechanisms:

### 1. Explicit Error Returns
- **[`Errors<T>`](`dada_ir_ast::diagnostic::Errors`)** type (alias for `Result<T, Reported>`)  
- **[`Reported`](`dada_ir_ast::diagnostic::Reported`)** token proves an error was already reported to user
- Tasks can fail independently without affecting other concurrent tasks

### 2. Embedded Error Values  
- **[`SymExpr::Err`](`crate::ir::exprs::SymExprKind`)** - Error expressions embedded in the IR
- **Infallible compilation** - System always produces some result, even with errors
- **Local error skipping** - Related work gets skipped via `?` operator when errors occur

### Error Recovery Strategy
- Expressions around errors can still compute normally
- Errors only propagate when something specifically inspects the failed expression
- Decision of what work to skip is optimized for user experience (currently ad-hoc)

## Future Work: LivePlaces

[`LivePlaces`](`crate::check::live_places::LivePlaces`) (currently unimplemented) will track which variables/places are live at each program point. This affects:

- **Subtyping with borrowing** - Types can reference places they borrow from
- **Move analysis** - Determining when values can be safely moved
- **Permission optimization** - More flexibility when variables won't be used again

Currently appears as `LivePlaces::fixme()` throughout the codebase.

## Implementation Location

Key modules:
- **[`crate::check::functions`]** - Function body checking entry points
- **[`crate::check::runtime`]** - Async runtime and constraint coordination  
- **[`crate::check::env`]** - Type checking environments and combinators
- **[`crate::check::inference`]** - Inference variable data structures
- **[`crate::check::red`]** - Reduced type representation and bounds