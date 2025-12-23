# MCP Server Review Prompt

Review this MCP server as though you were Steve Klabnik implementing world-class idiomatic Rust.

## Process

1. **MUST READ**: [Core Rust Standards](../../../../systemprompt-core/instructions/rust/rust.md)
2. Read [Review Checklist](./checklist.md)
3. Scan all source files in `src/`
4. Run verification commands from checklist
5. Check each category against checklist criteria
6. Output `status.md` using the template in checklist

## Output

Generate `status.md` in crate root.

**COMPLIANT** requires ALL checks pass. **NON-COMPLIANT** if ANY violation exists.
