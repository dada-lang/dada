# Constraint System Deep Dive

This document provides detailed information about Dada's type inference constraint system implementation.

## Direction Parameter

The [`Direction`](`crate::check::inference::Direction`) enum indicates whether a bound comes from above or below in the type hierarchy:

- **`FromBelow`**: Lower bound constraint (L <: ?X)
  - The inference variable must be at least L
  - Example: When `?X` appears in a position expecting a supertype
  
- **`FromAbove`**: Upper bound constraint (?X <: U)  
  - The inference variable must be at most U
  - Example: When `?X` appears in a position expecting a subtype

## Bound Representation

Constraints are stored as bounds on inference variables:

```rust
// For permissions
Perm {
    lower: Option<(RedPerm<'db>, ArcOrElse<'db>)>,
    upper: Option<(RedPerm<'db>, ArcOrElse<'db>)>,
}

// For types  
Ty {
    perm: InferVarIndex,  // Associated permission variable
    lower: Option<(RedTy<'db>, ArcOrElse<'db>)>,
    upper: Option<(RedTy<'db>, ArcOrElse<'db>)>,
}
```

Each bound is a tuple containing:
- **Bound value**: The actual type/permission bound ([`RedTy`](`crate::check::red::RedTy`) or [`RedPerm`](`crate::check::red::RedPerm`))
- **Error context** ([`ArcOrElse`](`crate::check::report::ArcOrElse`)): Explains why this bound was required

The `ArcOrElse` provides error context if the constraint cannot be satisfied. For example:
- If we add a lower bound `Int` because a variable was passed to a function expecting integers
- The associated `ArcOrElse` would explain "required because argument to function expecting Int"
- If later we find this conflicts with an upper bound `String`, we can report both requirements

## Constraint Storage and Updates

The [`Runtime`](`crate::check::runtime::Runtime`) maintains constraint state:
- `inference_vars`: All inference variable data with their bounds
- `sub_inference_var_pairs`: Direct subtype relationships between inference variables
- `waiting_on_inference_var`: Tasks blocked on specific variables

Constraints are added through centralized methods:
- [`Runtime::mutate_inference_var_data`](`crate::check::runtime::Runtime::mutate_inference_var_data`) - Only way to modify inference variables
- Ensures all waiting tasks are awakened when bounds change
- Maintains monotonicity - bounds only get tighter, never looser

## Inference Completion Detection

The system uses a two-phase execution model to determine when inference is complete:

### Phase 1: Run to Exhaustion
```rust
runtime.drain();  // Run all tasks until no more progress
```
- Executes all ready tasks
- Tasks either complete or block waiting for inference variables
- Continues until ready queue is empty

### Phase 2: Force Completion
```rust
runtime.mark_complete();  // Wake all blocked tasks
runtime.drain();          // Let them handle incomplete inference
```
- Sets `complete` flag and wakes ALL blocked tasks
- Tasks see `check_complete() == true` and must handle incomplete inference
- Allows reporting errors or using default values

### Main Task Tracking

The critical insight is tracking the **main task** separately:
```rust
let result = match channel_rx.try_recv() {
    Ok(v) => cleanup(v),     // Main task completed
    Err(_) => R::err(db, runtime.report_type_annotations_needed(span))
}
```

- **Main task**: The primary computation (e.g., building [`SymExpr`](`crate::ir::exprs::SymExpr`) for function body)
- **Background tasks**: Validation and constraint propagation

If the main task never completes (still blocked after `mark_complete`), it indicates insufficient type information in the program - for example:
- Cannot resolve method calls due to unknown receiver type
- Cannot determine which overload to use
- Cannot verify type bounds

This design distinguishes between:
- **Successful inference**: Main task completes (even with some validation errors)
- **Insufficient information**: Main task cannot even build the IR

## Constraint Propagation

Background tasks monitor and propagate constraints:

### `relate_infer_bounds` Task
- Monitors each inference variable
- When both upper and lower bounds exist, ensures `lower <: upper`
- Uses async iteration to watch for bound changes

### Subtype Relationships
- Direct relationships stored in `sub_inference_var_pairs`
- Propagated through transitive closure
- Awakens dependent tasks when new relationships discovered

## Error Handling in Constraints

Each constraint carries error context ([`ArcOrElse`](`crate::check::report::ArcOrElse`)) that explains why the constraint was required:
- Source location where constraint originated
- Reason for the requirement (e.g., "argument to function expecting Int")
- Used to build informative error messages when constraints conflict

When constraints conflict:
- Both requirements are available for error reporting
- System can explain why each bound was necessary
- Continues collecting other errors for comprehensive diagnostics