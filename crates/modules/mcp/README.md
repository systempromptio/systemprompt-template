# SystemPrompt MCP Module

Model Context Protocol (MCP) server management module for SystemPrompt.

## Overview

This module provides MCP server lifecycle management:
- MCP server registration and discovery
- Server process management (start/stop/restart)
- Port allocation and routing
- Health monitoring and status checking
- Tool usage tracking and analytics

## Database Usage

This module uses the `DatabaseProvider` abstraction for all database operations.

### Repository Pattern

```rust
use systemprompt_database::DatabaseProvider;
use std::sync::Arc;

pub struct McpRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl McpRepository {
    pub async fn get_server(&self, name: &str) -> Result<Option<McpServer>> {
        let row = self.db
            .fetch_optional(queries::GET_SERVER, &[&name])
            .await?;

        row.map(|r| McpServer::from_json_row(&r)).transpose()
    }
}
```

See [Database Migration Guide](../../../docs/database-migration-guide.md) for details.
