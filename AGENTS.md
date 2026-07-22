# opencodeR Agent Workflow

## Automatic repo issue tracking

At the start of each session and every 30 minutes, the agent:

1. Checks `github.com/grave0x/opencodeR` for open issues via `gh issue list`
2. Compares against the current TODO list
3. Adds any new issues as pending tasks with tag `github-issue-#N`
4. Reports summary

### Cron schedule

A recurring cron task fires every 30 minutes (`*/30 * * * *`). Manage it with:

- List: `/tasks` or `CronList`
- Cancel: `CronDelete <id>` (id: `d08257fc`)
- Modify: `CronCreate` with new schedule, then `CronDelete` the old one

### Manual trigger

Run `/check-issues` or invoke the `check-repo-issues` skill directly.

## Task conventions

- Each TODO list item maps to one deliverable
- Tags: `github-issue-#N` for repo issues, `feature-#N` for roadmap features
- Status transitions: pending → in_progress → done
- Blocked items note the blocker in the title

## Release workflow

Tag pushes (`v*`) trigger `.github/workflows/release.yml` which:
- Builds for Linux x86_64, Linux ARM64, Windows x86_64
- Creates .deb, .rpm, and Arch packages
- Uploads all artifacts to the GitHub Release
