---
name: "User Management"
description: "Manage users, roles, sessions, and IP bans via the systemprompt CLI"
---

# User Management

You manage users, roles, sessions, and IP bans using the systemprompt CLI. All operations go through the `admin users` domain.

## User CRUD

| Command | Purpose |
|---------|---------|
| `systemprompt admin users list` | List all users |
| `systemprompt admin users list --role admin` | List by role (admin, user, anonymous) |
| `systemprompt admin users list --status active` | List by status (active, inactive, suspended, pending, deleted, temporary) |
| `systemprompt admin users show <identifier>` | View user details |
| `systemprompt admin users show <identifier> --sessions` | View user with sessions |
| `systemprompt admin users show <identifier> --activity` | View user with activity |
| `systemprompt admin users search <query>` | Search users |
| `systemprompt admin users create --name <name> --email <email>` | Create a new user |
| `systemprompt admin users update <user-id> --email <email>` | Update user email |
| `systemprompt admin users update <user-id> --status suspended` | Suspend a user |
| `systemprompt admin users delete <user-id> -y` | Delete a user (irreversible) |
| `systemprompt admin users count` | Get total user count |
| `systemprompt admin users count --breakdown` | User count by role/status |
| `systemprompt admin users stats` | User statistics dashboard |
| `systemprompt admin users export -o users.json` | Export users to JSON |

## Role Management

| Command | Purpose |
|---------|---------|
| `systemprompt admin users role assign <user-id> --roles admin` | Assign roles to a user |
| `systemprompt admin users role promote <identifier>` | Promote user to admin |
| `systemprompt admin users role demote <identifier>` | Demote user from admin |

## Session Management

| Command | Purpose |
|---------|---------|
| `systemprompt admin users session list <user-id>` | List user's sessions |
| `systemprompt admin users session list <user-id> --active` | List active sessions only |
| `systemprompt admin users session end <session-id>` | End a specific session |
| `systemprompt admin users session end --user <user-id> --all -y` | End all user sessions |
| `systemprompt admin users session cleanup --days 30 -y` | Clean up old anonymous users |

## IP Ban Management

| Command | Purpose |
|---------|---------|
| `systemprompt admin users ban list` | List active IP bans |
| `systemprompt admin users ban add <ip> --reason "reason"` | Ban an IP address |
| `systemprompt admin users ban add <ip> --reason "reason" --permanent` | Permanent ban |
| `systemprompt admin users ban add <ip> --reason "reason" --duration 24h` | Temporary ban |
| `systemprompt admin users ban remove <ip> -y` | Remove an IP ban |
| `systemprompt admin users ban check <ip>` | Check if an IP is banned |
| `systemprompt admin users ban cleanup -y` | Clean up expired bans |

## Bulk Operations

| Command | Purpose |
|---------|---------|
| `systemprompt admin users bulk delete --status deleted --dry-run` | Preview bulk delete |
| `systemprompt admin users bulk delete --status deleted -y` | Execute bulk delete |
| `systemprompt admin users bulk update --set-status suspended --older-than 90d --dry-run` | Preview bulk status update |
| `systemprompt admin users merge --source <id> --target <id> -y` | Merge two user accounts |

## Standard Workflow

1. **List users** to see the current state before making changes
2. **Show user** to inspect a specific user's details, sessions, and activity
3. **Operate** -- create, update, assign roles, or manage sessions
4. **Verify** -- show the user again to confirm the change took effect

## Common Tasks

### Promote a User to Admin

```bash
systemprompt admin users show <identifier>
systemprompt admin users role promote <identifier>
systemprompt admin users show <identifier>
```

### Find and Ban a Suspicious IP

```bash
systemprompt admin users session list <user-id>
systemprompt admin users ban add <ip> --reason "Suspicious activity"
systemprompt admin users ban check <ip>
```

### Clean Up Inactive Users

```bash
systemprompt admin users session cleanup --days 30 -y
systemprompt admin users count --breakdown
```

### Investigate a User

```bash
systemprompt admin users search "<query>"
systemprompt admin users show <user-id> --sessions --activity
```

## Important Notes

- Deleting a user is irreversible -- confirm the user ID carefully
- Banning an IP does not end existing sessions -- end sessions separately
- Use `--dry-run` on bulk operations to preview before executing
- `--reason` is required when banning IPs
- Use `--help` on any subcommand for full flag reference
