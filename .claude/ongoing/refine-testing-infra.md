# Ongoing: Refine Testing Infrastructure for AI Assistants

## Current Status
**DESIGN PHASE** - Designing the solution and documentation before implementation

## Overall Goal
Improve the testing infrastructure to provide actionable, structured output that AI assistants can easily interpret and act upon, reducing confusion and debugging time.

## Problem Statement
Current testing output is optimized for human developers who understand codebase context, but is opaque to AI assistants:

- `cargo dada test` failures only show "Error: 1 test failure(s)" with no details
- No indication of which specific tests failed
- No guidance on failure type (needs blessing vs code error vs timeout)
- No actionable next steps provided

This leads to AI assistants:
- Running wrong commands from wrong directories
- Not understanding that failures often mean "bless references" not "fix code"
- Struggling to debug test issues that humans would quickly recognize

## Proposed Solution: `--porcelain` Flag

Add a `--porcelain` flag to `cargo dada test` that outputs structured, machine-readable results.

### JSON Output Format (DESIGN)
```json
{
  "summary": {
    "total": 3,
    "passed": 2, 
    "failed": 1
  },
  "tests": [
    {
      "path": "tests/parser/operator_precedence.dada",
      "status": "fail",
      "reason": "reference_mismatch",
      "suggestion": "UPDATE_EXPECT=1 cargo dada test tests/parser/operator_precedence.dada",
      "details": "AST output differs from .ref file"
    },
    {
      "path": "tests/hello_world.dada", 
      "status": "pass"
    },
    {
      "path": "tests/escape_sequences.dada",
      "status": "pass" 
    }
  ]
}
```

### Failure Reason Categories (DESIGN)
- **`reference_mismatch`**: Output differs from `.ref` file (needs blessing)
- **`compilation_error`**: Code fails to compile/parse
- **`runtime_error`**: Code compiles but crashes/fails during execution
- **`timeout`**: Test exceeded time limit
- **`missing_annotation`**: Required test annotation missing or invalid
- **`spec_validation_error`**: `#:spec` reference is invalid

## CLAUDE.md Documentation Design

The following section needs to be designed for CLAUDE.md to help AI assistants parse the new output:

### Testing Quick Reference (PROPOSED)
```markdown
### Testing Quick Reference  
- **Run tests**: `cargo dada test` (from repo root, not component dirs)
- **AI-friendly output**: `cargo dada test --porcelain` (structured JSON output)
- **Bless references**: `UPDATE_EXPECT=1 cargo dada test` (updates `.ref` files) 
- **Check test annotations**: Look at existing tests, don't invent (`#:skip_codegen`, `#:fn_asts`)

### Parsing Test Output
When using `--porcelain` flag:
- Parse JSON output from stdout
- Check `tests[].status` field: "pass" or "fail"
- For failures, check `reason` field:
  - `reference_mismatch`: Run the command in `suggestion` field (usually UPDATE_EXPECT)
  - `compilation_error` or `runtime_error`: Fix code issues
- Use `suggestion` field for exact command to run next
```

## Design Questions to Resolve

1. **JSON Schema**: Is the proposed format sufficient? Missing any fields?
2. **Failure Categories**: Are these the right categories? Missing any common failure types?
3. **CLAUDE.md Format**: Is the proposed documentation concise and actionable enough?
4. **Suggestion Format**: Should suggestions be exact commands or more descriptive?
5. **Error Details**: How much detail should the `details` field contain?

## Implementation Plan (AFTER DESIGN APPROVAL)

### Phase 1: Finalize Design
- Review and approve JSON schema
- Finalize failure reason categories  
- Approve CLAUDE.md documentation format

### Phase 2: Update Test Runner
- Add `--porcelain` flag to `cargo dada test`
- Modify test execution to capture detailed failure information
- Implement JSON output generation

### Phase 3: Update CLAUDE.md
- Add finalized testing reference section
- Document structured output parsing for AI assistants

### Phase 4: Validation
- Test with various failure scenarios
- Ensure suggestions are accurate and actionable
- Verify output is easily parseable by AI assistants

## Success Metrics
- AI assistants can immediately identify failing tests from JSON output
- AI assistants understand difference between blessing vs code fixes from `reason` field
- Reduced time spent debugging test infrastructure issues
- More reliable test execution in AI-assisted development