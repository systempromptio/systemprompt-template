# SystemPrompt Log Module

Logging and CLI display services for SystemPrompt.

## Overview

This module provides logging capabilities:
- Structured logging with context tracking
- Analytics-aware logging (session, trace, user context)
- CLI output formatting and styling
- Log streaming and querying
- Module-specific log filtering

## Database Usage

This module uses the `DatabaseProvider` abstraction for all database operations.

### Repository Pattern

```rust
use systemprompt_database::DatabaseProvider;
use std::sync::Arc;

pub struct LogRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl LogRepository {
    pub async fn get_logs(&self, module: &str) -> Result<Vec<LogEntry>> {
        let rows = self.db
            .fetch_all(queries::GET_LOGS, &[&module])
            .await?;

        rows.iter()
            .map(|r| LogEntry::from_json_row(r))
            .collect()
    }
}
```

See [Database Migration Guide](../../../docs/database-migration-guide.md) for details.
