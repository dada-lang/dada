---
name: tracking-issues
description: Track context across sessions for long-running features. Use when starting multi-session work, checkpointing progress, or resuming work on a feature tracked in a GitHub issue.
---

# Tracking Long-Running Work

Use GitHub issues as living documents to maintain context across work sessions. One issue per user-facing feature.

## Quick Reference

```bash
# Find active work
gh issue list --label tracking-issue

# Check a specific issue
gh issue view <number>
```

## When to Use

**Not for RFC features.** RFC-tracked work uses `impl.md` in the RFC directory for detailed progress tracking. A GitHub issue for an RFC should just be a lightweight pointer with links to the RFC and its impl status (e.g., `https://dada-lang.org/rfcs/NNNN-feature-name/impl.html`).

For non-RFC work (refactors, bug investigations, infrastructure) that spans 2+ sessions or multiple code areas, use a tracking issue.

## Issue Structure

**Labels**: `tracking-issue`, `ai-managed`, plus type (`feature`, `bug`, `refactor`)

**Title**: Clear user-facing outcome (not "encryption work" — instead "Implement client-side encryption")

**OP template** (keep updated as the living summary):

```markdown
# Feature Name

**Status**: Planning | In Progress | Blocked | Complete

## Current Understanding
Brief summary of what needs to be done and current approach

## Next Steps
- [ ] Specific actionable item with file:line references
- [ ] Another concrete next step

## Open Questions
- What we're still figuring out

## Context
Key background and why this work matters now
```

## Working with Issues

### Starting a session
Read the issue OP to understand current state. Work from "Next Steps."

### During work
- **Update OP** when: approach changes, major blockers discovered, next steps shift
- **Add comments** when: completing work sessions, discovering insights, hitting roadblocks

### Checkpointing
1. Find relevant tracking issue
2. Draft a comment summarizing the session
3. Show draft to user for approval before posting
4. Update OP if approach or next steps changed

### Comment structure

```markdown
**Session summary:**
- What was attempted or explored
- Key discoveries or problems encountered

**Impact on approach:**
- How understanding changed
- New questions that emerged

**Progress:** Completed items from next steps, what's next
```

### Completion
Set status to "Complete" and close the issue.

## Boundaries

- Only modify issues labeled `ai-managed`
- Always get user approval before posting comments or editing the OP
- Reference issues in commit messages when relevant
