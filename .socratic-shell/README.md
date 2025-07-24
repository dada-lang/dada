# Project Collaboration Patterns

This directory contains collaboration patterns designed to be installed per project.

The installation step imports these scripts into your repository and includes a script that can be used to synchronize the files in your repository with the "main copies" found in the github repository.

## Installation

```bash
curl https://raw.githubusercontent.com/socratic-shell/socratic-shell/main/src/prompts/project/install.sh | bash
```

This will sync the patterns to your project's `.socratic-shell/` directory.

## Usage

Add to your project's CLAUDE.md:

```markdown
# Team Collaboration Patterns
@.socratic-shell/README.md
```

## Files

- `README.md` - This file
- `install.sh` - Installation script for project teams
