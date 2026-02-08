# GitHub Tracking Issues

*Convention for tracking ongoing work using GitHub issues as living documents*

## Quick Start

**Check current work**: `gh issue list --label tracking-issue`
**Create new issue**: Get approval, use labels `tracking-issue,ai-managed,feature`
**During work**: Update OP for major changes, add comments for session details
**Checkpoint**: Draft comment with session progress, get approval before posting

## The Pattern

Use GitHub issues with the `tracking-issue` label to track active development work. One issue per user-facing feature that takes multiple work sessions. The Original Post (OP) serves as current status summary, while comments capture the detailed work journey.

**Scope guideline**: If it would take 2+ days or involves multiple code areas, it probably warrants a tracking issue.

## Issue Creation Convention

**Title**: Clear description of user-facing feature
- ✅ "Implement offline PWA support"
- ✅ "Add relationship calculator to family tree"
- ❌ "Encryption work" or "Improve codebase"

**Labels**:
- `tracking-issue` - Identifies ongoing work item
- `ai-managed` - Allows AI to update OP and add comments (without this label, AI should not modify the issue)
- Type labels: `feature`, `bug`, `architecture`, `refactor` as appropriate

**Initial OP Structure**:
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
- Dependencies on external decisions

## Context
Key background and why this work matters now
```

## Key Conventions

**OP as living summary**: Keep the Original Post updated to reflect current understanding - a fresh developer should read the OP and know exactly where things stand

**Comments for journey**: Use issue comments to document work sessions, discoveries, and how understanding evolved

**Update thresholds**:
- Update OP when: approach changes, major blockers discovered, next steps significantly different
- Add comments when: completing work sessions, discovering important insights, hitting roadblocks

**AI boundaries**: Only update issues labeled `ai-managed`, always get user approval before posting/editing anything

## Workflow Examples

**Starting work session**: Read issue OP to understand current state, work from "Next Steps"

**When user says "checkpoint our work"**:
1. Find relevant tracking issue (check `gh issue list --label tracking-issue`)
2. If no relevant issue exists, ask user if you should create one
3. Draft comment documenting the session (see structure below)
4. Show draft to user for approval before posting
5. Update OP if approach or next steps changed significantly

**Creating new tracking issue**:
1. Ask user for approval first
2. Use labels: `tracking-issue`, `ai-managed`, plus type (`feature`, `bug`, etc.)
3. Title should describe user-facing outcome
4. Fill OP template with current understanding

**Work completion**: Set status to "Complete", close issue after feature is deployed

## Content Guidelines

**OP contains** (always current):
- Current status and concrete next steps
- Open questions that still need resolution  
- Key context for understanding the work

**Comments contain** (historical journey):
- Work session summaries and discoveries
- Detailed progress updates and explorations
- Failed approaches and lessons learned

## Comment Structure

```markdown
**Session summary:**
- What was attempted or explored
- Key discoveries or problems encountered

**Impact on approach:**
- How understanding changed
- New questions that emerged

**Progress:** Completed items from next steps, what's next
```

**Example**:
```markdown
**Session summary:**
- Explored Web Crypto API for encryption
- Implemented basic key derivation with PBKDF2

**Impact on approach:**
- SubtleCrypto doesn't support extractable keys for our use case
- This breaks our planned multi-device sync approach
- Need to choose: extractable keys (security trade-off) vs device-specific keys (UX trade-off)

**Progress:** Completed key derivation research. Next: exploring device-specific keys approach.
```

## Integration with Development

**Reference in commits**:
```
Implement PBKDF2 key derivation for client encryption

Add basic key generation using Web Crypto API as first step
toward offline PWA support. See progress in issue #47.
```

**Related work**: Reference other issues when dependencies emerge, always discuss with user before creating new tracking issues

## Benefits

- **Context preservation**: No mental reload between sessions
- **Team visibility**: Current state and journey both visible
- **Decision tracking**: Rationale for choices stays accessible
- **Natural workflow**: Uses familiar GitHub issue patterns

## AI Guidelines

- Read OP first to understand current state, review recent comments for context
- Only modify issues labeled `ai-managed`
- Always get user approval before posting comments or editing OP
- Focus OP on current status, use comments for session details