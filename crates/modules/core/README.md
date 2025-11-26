# SystemPrompt Core Module

Core functionality and shared services for SystemPrompt.

## Overview

This module provides core system functionality:
- Service registry and management
- Analytics and tracking
- System health monitoring
- Process monitoring
- Database validation and setup
- Bootstrap and initialization

## Database Usage

This module uses the `DatabaseProvider` abstraction for all database operations.

### Repository Pattern

```rust
use systemprompt_database::DatabaseProvider;
use std::sync::Arc;

pub struct ServiceRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl ServiceRepository {
    pub async fn get_service(&self, id: &str) -> Result<Option<Service>> {
        let row = self.db
            .fetch_optional(queries::GET_SERVICE, &[&id])
            .await?;

        row.map(|r| Service::from_json_row(&r)).transpose()
    }
}
```

See [Database Migration Guide](../../../docs/database-migration-guide.md) for details.
