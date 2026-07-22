# opencodeR Agentic Workflow

This file defines how Kimi Code should handle issues, PRs, and feature requests
for the opencodeR project.

## Issue workflow

When a new GitHub issue is created:

1. The **triage workflow** (`.github/workflows/triage.yml`) auto-labels the issue
   based on keywords in the title.
2. If a project board number is set in `OPENCODE_PROJECT_NUMBER` repo variable,
   the issue is automatically added to the project board.
3. First-time contributors get a welcome comment.

When interacting with this repository, Kimi Code should:

- **Read new issues** from the GitHub API when the user asks about project status
- **Auto-create TodoList entries** for new feature requests or bug reports
- **Prioritize P0/P1 issues** over feature development
- **Reference issue numbers** in commit messages (e.g. `feat: close #12`)
- **Update feature tracking** in `features/features.json` when implementing roadmap items

## Agentic todo sync

When the user says "check repo for issues" or similar:

1. Run `gh issue list --repo grave0x/opencodeR --state open -L 20`
2. For each open issue:
   - If it's a bug report (`[bug]`): create a TodoList entry with priority
   - If it's a feature request (`[feat]`): check against `features/ROADMAP.md`
     and create a TodoList entry if it's not already tracked
   - If it's a question: answer it directly
3. Report the summary to the user

## Commit message convention

```
type(scope): description

- feat: new feature
- fix: bug fix
- chore: maintenance, deps, CI
- docs: documentation
- refactor: code restructuring
- perf: performance optimization
- feat: close #N — references and closes an issue
```

## Release workflow

The release workflow (`.github/workflows/release.yml`) triggers on `v*` tags.
It builds all three binaries for Linux x86_64, Linux ARM64, and Windows x86_64,
creates tarballs, .deb packages, and uploads to the GitHub Release.
