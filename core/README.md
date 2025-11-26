# SystemPrompt Core

**Platform and framework for SystemPrompt OS**

## What is SystemPrompt Core?

SystemPrompt Core provides the foundational platform for building AI agent orchestration systems:

- **Agent Orchestration**: Multi-agent lifecycle management via A2A protocol
- **MCP Protocol**: Model Context Protocol server and client implementations
- **Database Abstraction**: Unified interface supporting SQLite and PostgreSQL
- **HTTP API Framework**: REST API with OAuth2 authentication
- **Module System**: Extensible module architecture with schema management
- **Configuration**: Multi-environment configuration system
- **Structured Logging**: Context-aware logging with request tracing

## Architecture

Core uses a **module-based architecture**:

```
crates/
├── shared/           # Shared libraries
│   ├── models/      # Data models and types
│   ├── traits/      # Trait definitions
│   └── identifiers/ # ID types
└── modules/         # Core platform modules
    ├── agent/       # Agent orchestration (A2A protocol)
    ├── mcp/         # MCP server management
    ├── api/         # HTTP API server
    ├── database/    # Database abstraction
    ├── oauth/       # OAuth2 authentication
    ├── users/       # User management
    ├── log/         # Logging system
    ├── ai/          # AI integrations
    ├── rag/         # RAG functionality
    └── config/      # Configuration
```

Each module owns its schema via `module.yaml`.

## Distribution Model

**SystemPrompt Core is distributed via Git (not crates.io).**

This workspace contains 14 interdependent crates that share a single version and are published together as one repository. This approach ensures version consistency and simplifies the development experience for a tightly-coupled platform.

**Why Git?**
- ✅ All 14 crates versioned together (impossible to have mismatches)
- ✅ Simple publishing (one git tag vs 14 separate publications)
- ✅ Workspace structure preserved (path dependencies work naturally)
- ✅ Private repository support

See [FAQ](./docs/FAQ.md#why-git-dependencies-instead-of-cratesio) for detailed explanation.

## Installation

### Quick Start

Add to your `Cargo.toml`:

```toml
[workspace.dependencies]
# Import only the modules you need
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core", tag = "v0.1.0" }
systemprompt-core-api = { git = "https://github.com/systempromptio/systemprompt-core", tag = "v0.1.0" }
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core", tag = "v0.1.0" }
systemprompt-core-mcp = { git = "https://github.com/systempromptio/systemprompt-core", tag = "v0.1.0" }
systemprompt-core-agent = { git = "https://github.com/systempromptio/systemprompt-core", tag = "v0.1.0" }
```

Then in your service crate:

```toml
[dependencies]
systemprompt-models.workspace = true
systemprompt-core-api.workspace = true
```

Build your project:

```bash
cargo build
```

**First build**: 2-5 minutes (downloads repository ~50MB)
**Subsequent builds**: <30 seconds (uses cache)

### Optional: GeoIP Database Setup

For geographic analytics (IP geolocation), download the GeoIP database:

```bash
# From repository root
./scripts/download-geoip.sh
```

This downloads the free DB-IP database (~60MB, MaxMind-compatible). Without this file:
- Application runs normally
- Geographic data (country/region/city) will not be available in analytics
- Warning message shown in logs: `⚠ Warning: Could not load GeoIP database`

**Production**: Automatically downloaded by deployment scripts (`infrastructure/environments/production/terraform/startup.sh`)

📖 **[Complete Installation Guide](./docs/INSTALLATION.md)**

### For Core Contributors

```bash
# Clone repository
git clone https://github.com/systempromptio/systemprompt-core
cd systemprompt-core

# Build all modules
cargo build --workspace

# Run tests
cargo test --workspace

# Build specific crate
cargo build -p systemprompt-core-api

# Run CLI
cargo run --bin systemprompt -- --help
```

## Available Crates

| Crate | Description |
|-------|-------------|
| `systemprompt-models` | Shared data models and types |
| `systemprompt-identifiers` | ID types and generators |
| `systemprompt-traits` | Trait definitions |
| `systemprompt-core-database` | Database abstraction (SQLite + PostgreSQL) |
| `systemprompt-core-config` | Configuration management |
| `systemprompt-core-api` | HTTP API server framework |
| `systemprompt-core-mcp` | MCP protocol implementation |
| `systemprompt-core-agent` | Agent orchestration (A2A) |
| `systemprompt-core-oauth` | OAuth2 authentication |
| `systemprompt-core-users` | User management |
| `systemprompt-core-logging` | Structured logging |
| `systemprompt-core-ai` | AI service integrations |
| `systemprompt-core-rag` | RAG functionality |

## Code Quality & Standards

SystemPrompt maintains **world-class code quality** through automated tooling and strict standards.

### Style Enforcement

```bash
# Format code
just fmt

# Run all style checks (format + lint + validate)
just style-check
```

**Our Standards**:
- ✅ Self-documenting code (no inline comments)
- ✅ Low cognitive complexity (max 15)
- ✅ Short functions (max 100 lines)
- ✅ Type-safe patterns (DatabaseQueryEnum, repository pattern)
- ✅ Comprehensive error handling (no .unwrap())

📖 **[Complete Coding Standards](./tests/validator/coding-standards.md)**

### Automated Checks

Every pull request runs:
1. **Rustfmt** - Code formatting
2. **Clippy** - Linting with strict rules
3. **Custom Validators** - SystemPrompt-specific patterns

See [tests/validator/README.md](./tests/validator/README.md) for details.

## Module System

Each module has a `module.yaml` defining:
- Schema files (SQL)
- Permissions
- Dependencies
- API configuration
- Commands

Example: `crates/modules/agent/module.yaml`

Modules are loaded dynamically at runtime.

## Schema Management

Schemas are **module-owned** and referenced via `module.yaml`:

```yaml
schemas:
  - file: "schema/agent_tasks.sql"
    table: "agent_tasks"
    required_columns: ["id", "uuid"]
```

Schema files remain in each module's directory.

## Versioning

Follows [Semantic Versioning](https://semver.org/):

- **Major** (1.0.0): Breaking API changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, backward compatible

## Documentation

### Guides

- **[Architecture Guide](./docs/ARCHITECTURE.md)** - Workspace structure, module organization, and design principles
- **[Installation Guide](./docs/INSTALLATION.md)** - Detailed installation instructions and version management
- **[Dependency Guide](./docs/DEPENDENCY_GUIDE.md)** - Technical deep dive on how dependencies work
- **[Publishing Guide](./docs/PUBLISHING.md)** - How to release new versions (for maintainers)
- **[FAQ](./docs/FAQ.md)** - Frequently asked questions

### Additional Documentation

- [Module System](./CLAUDE.md#module-structure) - How modules are organized
- [Database Patterns](./CLAUDE.md#repository--database-patterns) - Repository and query patterns
- **[Coding Standards](./tests/validator/coding-standards.md)** - Rust style guide and best practices
- **[Style Quick Reference](./tests/validator/QUICK_REFERENCE.md)** - One-page style reference
- [Contributing Guide](./CONTRIBUTING.md) - (coming soon)
- [Changelog](./CHANGELOG.md) - (coming soon)

## License

MIT License - see [LICENSE](LICENSE) for details

## Links

- [GitHub Repository](https://github.com/systempromptio/systemprompt-core)
- [Issues](https://github.com/systempromptio/systemprompt-core/issues)
- [Discussions](https://github.com/systempromptio/systemprompt-core/discussions)
