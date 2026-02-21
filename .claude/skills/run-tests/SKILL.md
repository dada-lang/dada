---
name: run-tests
description: Run and interpret Dada test results. Use when running tests, debugging test failures, or understanding test output.
---

# Running Dada Tests

## Commands

```bash
# Run all tests
cargo dada test --porcelain

# Run tests in a directory
cargo dada test --porcelain tests/syntax/string_literals/

# Run a single test file
cargo dada test --porcelain tests/syntax/string_literals/type.dada
```

Always use `--porcelain` for machine-readable JSON output with structured failure information.

## Interpreting Output

The `--porcelain` flag produces JSON with this structure:

```json
{
  "summary": { "total": 45, "passed": 45, "failed": 0, "duration_ms": 70 },
  "tests": [
    {
      "path": "tests/syntax/string_literals/type.dada",
      "status": "pass",
      "annotations": ["#:skip_codegen", "#:spec syntax.string-literals.type"]
    }
  ]
}
```

For failures, each test includes:
- `suggestion` — Actionable guidance on how to resolve the failure. **Read this first.**
- `details` — Usually points to a `.test-report.md` file alongside the test

## Reading Test Reports

When a test fails, a `.test-report.md` file is generated next to the test file. It contains:

1. **Compiler output** — The actual errors/warnings produced
2. **Unexpected diagnostics** — Errors the compiler produced that the test didn't expect
3. **Missing expected diagnostics** — `#!` annotations that didn't match any compiler output
4. **Probe failures** — `#?` probes that got unexpected results
5. **Next Steps** — Suggestions for resolution

## Test Annotation Reference

### Spec references
```dada
#:spec syntax.string-literals.escape-sequences.backslash
```
Links the test to a spec paragraph. Validated against actual spec files.

### Directives
```dada
#:skip_codegen    # Skip WebAssembly generation (use for parser/type-check only tests)
#:fn_asts         # Compare function AST output against .ref file
```

### Diagnostic expectations (`#!`)

**Without carets** — error can start anywhere on the previous interesting line:
```dada
print(unknown_var)
#! could not find anything named `unknown_var`
```

**With carets** — error span must match exactly (caret position = column on previous line):
```dada
fn test() { bad_name() }
#!          ^^^^^^^^ could not find anything named `bad_name`
```

**With regex** — use `/` prefix (opening `/` only, NO closing `/`):
```dada
is_shared(x.mut)
#! /where clause.*not satisfied
```

> **Important**: The regex convention uses `/pattern` with NO closing `/`. Including a closing `/` causes the `/` to be part of the regex pattern, which will fail to match.

### Type probes (`#?`)

```dada
let x = 22 + 44
#?      ^^ ExprType: u32       # Type of expression at that span
#?  ^ VariableType: u32        # Type of the variable
```

Caret position must align with the target on the previous line. `VariableType` shows the declared type (e.g., `String` not `my String` — the permission is on the expression, not the variable).

Probes also support regex with `/`:
```dada
#?  ^ VariableType: /my.*String
```

## Common Patterns

- Tests that only exercise parsing/type-checking should use `#:skip_codegen`
- Error tests use `#!` annotations; the test passes when all expected errors match and no unexpected errors appear
- Probe tests use `#?` annotations; the test passes when all probes return expected values
