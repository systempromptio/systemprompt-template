# MCP Infrastructure Instructions

## Quick Reference

| Topic | Document |
|-------|----------|
| **Architecture** | |
| Overview | [architecture/overview.md](./architecture/overview.md) |
| Boundaries | [architecture/boundaries.md](./architecture/boundaries.md) |
| **Implementation** | |
| Tools | [implementation/tools.md](./implementation/tools.md) |
| Progress Reporting | [implementation/progress.md](./implementation/progress.md) |
| Skills Integration | [implementation/skills.md](./implementation/skills.md) |
| Error Handling | [implementation/errors.md](./implementation/errors.md) |
| Prompts | [implementation/prompts.md](./implementation/prompts.md) |
| **Configuration** | |
| Profiles | [config/profiles.md](./config/profiles.md) |
| Services | [config/services.md](./config/services.md) |
| **Quality** | |
| Review Prompt | [review/prompt.md](./review/prompt.md) |
| Review Checklist | [review/checklist.md](./review/checklist.md) |

---

## Core Rules

All code MUST comply with [systemprompt-core Rust Standards](../../../systemprompt-core/instructions/rust/rust.md).

Key rules:

| Rule | Description |
|------|-------------|
| Zero comments | No `//`, no `///`, no `//!` |
| No panics | No `unsafe`, `unwrap()`, `panic!()`, `todo!()` |
| File limits | 300 lines max, 75 line functions, 5 parameters |
| Typed identifiers | `McpServerId`, `McpExecutionId`, `SkillId` |
| Error handling | `thiserror` enums, `anyhow` at boundaries only |
| Logging | `tracing` with structured fields |
