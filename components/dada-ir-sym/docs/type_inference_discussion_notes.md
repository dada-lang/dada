# Type Inference Discussion Checklist

This document tracks topics for future discussions about Dada's type inference system.

## Topics Covered ✅

### Architecture & Flow
- ✅ Overall architecture (AST → Symbolic IR during function checking)
- ✅ Salsa query-driven compilation (demand-driven, not strict phases)  
- ✅ Function-scoped inference (variables don't cross boundaries)
- ✅ Runtime::execute creating async execution environment

### Concurrency Model
- ✅ Async/await for constraint coordination (not I/O)
- ✅ Structured concurrency (`future::join`) vs background tasks (spawn)  
- ✅ Environment forking for task isolation with shared constraint state
- ✅ Concrete example: `check_call_common` parallel argument checking

### Inference Variables  
- ✅ Creation via `Env::fresh_inference_var()` with `SymGenericKind`
- ✅ Type variables automatically get associated permission variables
- ✅ Background constraint validation tasks (relate_infer_bounds, reconcile_ty_bounds)
- ✅ No place inference variables (not planned)

### Constraint System
- ✅ Red (reduced) types: factored into permission + core type components
- ✅ Monitor pattern with async/await for constraint coordination
- ✅ Monotonic bounds (only get tighter, never looser)
- ✅ Centralized updates via `Runtime::mutate_inference_var_data`
- ✅ Standardized waiting patterns (`loop_on_inference_var`)

### Error Handling
- ✅ Two-pronged approach: `Errors<T>` returns + embedded error values
- ✅ Fail-soft compilation (always produces some result)
- ✅ `Reported` token proves error was already shown to user
- ✅ Local error skipping via `?` operator, ad-hoc but user-experience focused

### Future Work
- ✅ LivePlaces overview (liveness analysis for borrowing and move analysis)

## Topics for Future Discussion ❓

### Constraint System Deep Dive
- ❓ **Direction parameter**: What does `Direction` represent? (Upper vs lower bounds? Variance?)
- ❓ **Bound representation**: What do the actual constraint bounds look like internally?
- ❓ **Constraint composition**: How do you coordinate multiple inference variables being ready?
- ❓ **RedBound methods**: Details of `RedBound::set_ty`, `RedBound::set_perm` 
- ❓ **Constraint conflicts**: How are circular dependencies detected/prevented?
- ❓ **Termination conditions**: How does the system know inference is complete?

### Inference Variable Lifecycle
- ❓ **Resolution timing**: When/how do variables get resolved to concrete types?
- ❓ **Constraint addition**: How are new constraints added beyond initial creation?
- ❓ **Variable substitution**: How does the final substitution phase work?
- ❓ **Garbage collection**: When/how are inference variables cleaned up?

### Bidirectional Inference Details
- ❓ **Information flow**: Concrete examples of bidirectional constraint propagation
- ❓ **Expected vs actual types**: How does "expected type flows down" actually work?
- ❓ **Conflict resolution**: What happens when upward and downward information conflicts?

### Permission Inference Specifics  
- ❓ **Permission bounds**: How do permission constraints differ from type constraints?
- ❓ **Permission lattice**: How is the permission subtyping lattice implemented?
- ❓ **Default permissions**: When/how are default permissions inferred?
- ❓ **Permission conflicts**: Examples of permission inference failures

### Advanced Features
- ❓ **Generic instantiation**: How does inference work with generic functions/types?
- ❓ **Method resolution**: How does inference interact with method dispatch?
- ❓ **Subtyping integration**: How do subtyping rules interact with inference?
- ❓ **Where clauses**: How are generic bounds handled during inference?

### Debugging & Observability
- ❓ **Task logging**: How does the debugging/logging system work?
- ❓ **Inference visualization**: Tools for understanding inference progress?
- ❓ **Performance monitoring**: How to identify inference bottlenecks?

### Error Messages & User Experience
- ❓ **Constraint conflicts**: What do type inference error messages look like?
- ❓ **Error recovery heuristics**: Guidelines for what work to skip on errors?
- ❓ **Multiple errors**: How are multiple inference failures presented to users?

### Implementation Details
- ❓ **Memory management**: How are inference contexts and variables stored?
- ❓ **Performance optimizations**: Strategies for fast constraint solving?
- ❓ **Incremental recompilation**: How does Salsa caching interact with inference?

## Discussion Notes Template

When discussing a topic, use this template:

### Topic: [Name]
**Date**: [Date]  
**Covered**: [Key points discussed]  
**Examples**: [Code examples or concrete scenarios]  
**Implementation**: [Relevant code locations/methods]  
**Questions raised**: [New questions that emerged]  
**Next steps**: [Follow-up topics]