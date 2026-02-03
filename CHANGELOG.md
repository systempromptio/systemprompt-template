# Changelog

All notable changes to systemprompt-template will be documented in this file.

## [0.1.0] - 2026-02-03

### Added
- Initial public release of systemprompt-template
- Web extension with blog, documentation, homepage, and playbook providers
- Soul extension with memory service and Discord integration
- MCP servers: systemprompt CLI, soul memory, content-manager
- Discord CLI binary for gateway management
- Comprehensive playbook system for operational guidance
- Agent configurations (welcome, assistant)
- Scheduler jobs for content analytics, llms.txt generation, and publishing

### Fixed
- All clippy warnings resolved across all extensions
- Fixed `unwrap_or_else` to `unwrap_or_default` patterns
- Fixed uninlined format args in blog renderers
- Added missing semicolons in closure expressions
- Added appropriate `#[allow]` attributes for intentional casts and long functions

### Technical
- Workspace-level clippy lints (deny all + pedantic)
- SQLx compile-time query validation support
- Full async/await patterns with tokio runtime
- Modular extension architecture with inventory-based discovery
