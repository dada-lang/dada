# Ongoing Work Tracking

*Convention for maintaining development context between sessions*

## The Pattern

Create `.ongoing/task-name.md` files to track active development work. One file per logical feature - the "big things I am working on right now". Multiple ongoing files can exist simultaneously for different features. These living documents evolve as understanding grows and enable easy work resumption.

## File Naming Convention

```
.ongoing/
├── feature-user-authentication.md
├── bug-memory-leak-parser.md  
├── refactor-database-layer.md
└── config-restructure.md
```

Use descriptive names that capture the work's essence. Prefix with type when helpful (feature-, bug-, refactor-, etc.).

## Essential Content Structure

```markdown
# Task Name

**Status**: Planning | In Progress | Blocked | Complete  
**Started**: YYYY-MM-DD  
**Goal**: One sentence describing success

## Current State
Brief context of where things stand right now

## Next Steps
- [ ] Specific actionable item with file:line references
- [ ] Another concrete next step
- [ ] etc.

## Blockers
(Only include this section when status is Blocked)
- Concrete external dependency preventing progress
- Who/what needs to resolve it

## Open Questions
- What approach for handling edge case X?
- Need to decide between option A vs B

## Context & Decisions
Key background info and why certain choices were made
```

## Status Definitions

- **Planning**: Designing approach, gathering requirements
- **In Progress**: Actively implementing 
- **Blocked**: Cannot proceed due to external dependency (identify the concrete blocker)
- **Complete**: Ready to delete file

## Key Conventions

**Real-time updates**: Update the file as work progresses - after completing each next step, making discoveries, or at natural pause points

**Specific next steps**: Include file paths and line numbers where possible
- ❌ "Fix the validation logic" 
- ✅ "Update validateUser() in src/auth.ts:42 to handle empty email case"

**Preserve decision context**: Capture not just what was decided, but why - prevents re-litigating settled questions

**Living evolution**: Move completed next steps to "Context & Decisions", add new discoveries, update status and current state

**File lifecycle**: Delete the file when work is complete (after feature is merged/deployed, not just when code is written)

## Git Tracking

Follow your project's existing pattern for `.ongoing/` files:
- If other `.ongoing/` files are committed → commit yours
- If they're gitignored → ignore yours  
- If unclear, ask the project maintainer

## Workflow Example

**Starting new logical feature**:
```bash
# 1. Create .ongoing/feature-name.md with template
# 2. Set status to "Planning", fill in goal
# 3. Add initial next steps
# 4. Begin implementation
```

**During development session**:
```bash
# 1. Read .ongoing/feature-name.md to reload context
# 2. Work from "Next Steps" list
# 3. Update file as you complete items:
#    - Move completed steps to "Context & Decisions"
#    - Add new next steps as they emerge
#    - Update "Current State" with progress
```

**Session completion**:
```bash
# 1. Update "Current State" with where you left off
# 2. Refine "Next Steps" for next session
# 3. Document any new discoveries or decisions
```

**Work completion**:
```bash
# 1. Set status to "Complete" 
# 2. After feature is merged/deployed, delete the file 
#    (context is preserved in git history and commit messages)
```

## Integration with Commits

Reference ongoing files in commit messages to show larger context:

```
Add user input validation to login form

Implement email format checking and required field validation 
as the first step toward secure authentication, per the plan 
in .ongoing/feature-user-authentication.md
```

This creates traceability between individual commits and the broader feature work.

## Benefits

- **Context preservation**: No mental reload time between sessions
- **Handoff ready**: Team members can pick up work easily  
- **Decision tracking**: Why choices were made stays visible
- **Progress visibility**: Status and next steps always current
- **Commit clarity**: Larger context visible in commit messages