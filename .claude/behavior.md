# Claude-Specific Behavior Guidelines

## Core Behavior Pattern

**ALWAYS check CLAUDE.md and .development/ FIRST** before analyzing the codebase or answering questions about established patterns.

## When You Should Update Documentation

### Discovering New Patterns
- Code conventions not documented in `.development/patterns.md`
- Architecture decisions not covered in `.development/architecture.md`
- Development workflows not in `.development/workflows.md`
- Documentation practices not in `.development/documentation.md`

### Making Implementation Decisions
- Update `.development/decisions/` with RFC-style decision records
- Capture rationale for future reference
- Include concrete examples and alternatives considered

## Multi-Session Work Tracking

Use `.claude/ongoing/` for work spanning multiple sessions:

### File Naming
`ongoing_feature_name.md` - One file per major task

### Required Content
- **Status**: Current state (planning, implementing, testing, etc.)
- **Context**: Background and motivation
- **Work Completed**: What's been done so far
- **Next Steps**: Specific tasks to continue
- **Code References**: File paths and line numbers for relevant code

### Lifecycle
- **Create** when starting multi-session work
- **Update** at end of each session
- **Remove** when work is complete

## Interaction Guidelines

- Reference specific files and line numbers when discussing code
- Use format `file_path:line_number` for easy navigation
- Check existing documentation before providing explanations
- Update documentation when discovering gaps or new patterns