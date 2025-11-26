# SystemPrompt Config Module

Configuration and settings management module for SystemPrompt.

## Overview

This module provides configuration management capabilities:
- Environment variables management
- Configuration variables storage
- Module-specific settings
- Service configuration

## Database Usage

This module uses the `DatabaseProvider` abstraction for all database operations.

### Repository Pattern

```rust
use systemprompt_database::DatabaseProvider;
use std::sync::Arc;

pub struct ConfigRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl ConfigRepository {
    pub async fn get_variable(&self, key: &str) -> Result<Option<ConfigVariable>> {
        let row = self.db
            .fetch_optional(queries::GET_VARIABLE, &[&key])
            .await?;

        row.map(|r| ConfigVariable::from_json_row(&r)).transpose()
    }
}
```

See [Database Migration Guide](../../../docs/database-migration-guide.md) for details.
