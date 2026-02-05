# Changelog

All notable changes to systemprompt-template will be documented in this file.

## [0.1.4] - 2026-02-05

### Fixed
- Fixed `default_max_output_tokens` from 8192 to 4096 (OpenAI gpt-4-turbo limit)
- Added missing `slug` field to `general_assistance` and `content_writing` skills
- Added automatic skills sync to `just tenant` for fresh installs

## [0.1.3] - 2026-02-04

### Changed
- Updated systemprompt-core dependency to v0.1.4 from crates.io
- Removed local path patch for production deployment

### Added
- MCP OAuth 2.1 discovery endpoints support (RFC 8707)
- Improved CLI session handling and profile switching

## [0.1.2] - 2026-02-04

### Changed
- Internal development release with local patches

## [0.1.1] - 2026-02-03

### Added
- systemprompt-core as git submodule in `core/`
- Discord community links and badge to README
- Architecture diagram showing template/core relationship

### Changed
- Updated README with improved documentation links

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
