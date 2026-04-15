---
name: "Job Scheduling"
description: "List, run, enable, and disable scheduled jobs via the systemprompt CLI"
---

# Job Scheduling

You manage scheduled background jobs using the systemprompt CLI. All operations go through the `infra jobs` domain.

## Available Commands

| Command | Purpose |
|---------|---------|
| `systemprompt infra jobs list` | List all jobs with cron schedule |
| `systemprompt infra jobs show <job-name>` | Show job details |
| `systemprompt infra jobs run <job-name>` | Run a job manually |
| `systemprompt infra jobs run --all` | Run all jobs |
| `systemprompt infra jobs run <job-name> -p key=value` | Run with parameters |
| `systemprompt infra jobs run <job1> <job2> --sequential` | Run multiple jobs in sequence |
| `systemprompt infra jobs run --tag <tag>` | Run all jobs with a tag |
| `systemprompt infra jobs history` | View execution history |
| `systemprompt infra jobs history --job <job-name>` | History for a specific job |
| `systemprompt infra jobs history --status failed` | Failed executions only (success, failed, running) |
| `systemprompt infra jobs history --limit 20` | Limit history entries |
| `systemprompt infra jobs enable <job-name>` | Enable a scheduled job |
| `systemprompt infra jobs disable <job-name>` | Disable a scheduled job |

## Built-In Jobs

| Job | Purpose | Flags |
|-----|---------|-------|
| `cleanup-sessions` | Clean up inactive sessions | `--hours` (default 1), `--dry-run` |
| `log-cleanup` | Remove old log entries | `--days` (default 30), `--dry-run` |

## Standard Workflow

1. **List jobs** to see all available jobs and their schedules
2. **Show job** to inspect details and configuration
3. **Check history** to see when a job last ran and whether it succeeded
4. **Operate** -- run manually, enable, or disable
5. **Verify** -- check history again to confirm execution

## Common Tasks

### View All Jobs and Schedules

```bash
systemprompt infra jobs list
```

### Run a Job Manually

```bash
systemprompt infra jobs run <job-name>
systemprompt infra jobs history --job <job-name> --limit 5
```

### Disable a Job Temporarily

```bash
systemprompt infra jobs disable <job-name>
systemprompt infra jobs list
```

### Check Job Health

```bash
systemprompt infra jobs list
systemprompt infra jobs history --status failed --limit 10
```

### Run Session Cleanup with Preview

```bash
systemprompt infra jobs run cleanup-sessions -p hours=24 --dry-run
systemprompt infra jobs run cleanup-sessions -p hours=24
```

## Important Notes

- Jobs run on cron schedules defined in `services/scheduler/`
- Running a job manually does not affect its scheduled runs
- Disabling a job prevents scheduled execution but allows manual runs
- Check history to verify successful completion after manual runs
- Use `--dry-run` on cleanup jobs to preview before executing
- Use `--help` on any subcommand for full flag reference
