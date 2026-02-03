---
title: "User Management Playbook"
description: "Manage users, roles, sessions, and IP bans."
author: "SystemPrompt"
slug: "cli-users"
keywords: "users, roles, sessions, admin, bans"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# User Management Playbook

Manage users, roles, sessions, and IP bans.

---

## List Users

```json
{ "command": "admin users list" }
{ "command": "admin users list --limit 50" }
{ "command": "admin users list --limit 20 --offset 40" }
```

---

## Search Users

```json
{ "command": "admin users search \"user@example.com\"" }
{ "command": "admin users search \"john\"" }
```

---

## Show User Details

```json
{ "command": "admin users show user_abc123" }
```

---

## Create User

```json
{ "command": "admin users create --email user@example.com --name \"New User\"" }
```

---

## Update User

```json
{ "command": "admin users update user_abc123 --name \"Updated Name\"" }
{ "command": "admin users update user_abc123 --email user@example.com" }
```

---

## Delete User

```json
{ "command": "admin users delete user_abc123" }
```

---

## User Count and Statistics

```json
{ "command": "admin users count" }
{ "command": "admin users stats" }
```

---

## Export Users

```json
{ "command": "admin users export" }
{ "command": "admin users export --format json" }
```

---

## Role Management

### Assign Roles

```json
{ "command": "admin users role assign user_abc123 admin" }
{ "command": "admin users role assign user_abc123 admin,editor" }
```

### Promote / Demote

```json
{ "command": "admin users role promote user_abc123" }
{ "command": "admin users role demote user_abc123" }
```

---

## Session Management

```json
{ "command": "admin users session list user_abc123" }
{ "command": "admin users session end session_xyz789" }
{ "command": "admin users session cleanup --hours 24" }
```

---

## IP Ban Management

### List Bans

```json
{ "command": "admin users ban list" }
```

### Add Ban

```json
{ "command": "admin users ban add 192.168.1.100 --reason \"Abuse\"" }
{ "command": "admin users ban add 192.168.1.100 --duration 1440 --reason \"Spam\"" }
```

Duration is in minutes (1440 = 24 hours).

### Remove / Check Ban

```json
{ "command": "admin users ban remove 192.168.1.100" }
{ "command": "admin users ban check 192.168.1.100" }
```

---

## Merge Users

Merge source user into target (combine accounts):
```json
{ "command": "admin users merge source_user_id target_user_id" }
```

---

## Bulk Operations

```json
{ "command": "admin users bulk delete --inactive-days 365" }
{ "command": "admin users bulk update --role user --new-role member" }
```

---

## Quick Reference

| Task | Command |
|------|---------|
| List users | `admin users list` |
| Search | `admin users search "query"` |
| Show user | `admin users show <id>` |
| Create user | `admin users create --email <email>` |
| Delete user | `admin users delete <id>` |
| User count | `admin users count` |
| Assign role | `admin users role assign <id> <role>` |
| Promote | `admin users role promote <id>` |
| Demote | `admin users role demote <id>` |
| List sessions | `admin users session list <id>` |
| Cleanup sessions | `admin users session cleanup --hours 24` |
| List bans | `admin users ban list` |
| Add ban | `admin users ban add <ip> --reason "..."` |
| Remove ban | `admin users ban remove <ip>` |