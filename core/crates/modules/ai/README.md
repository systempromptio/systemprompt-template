# SystemPrompt AI Module

AI provider integration and agentic execution module for SystemPrompt.

## Overview

This module provides AI capabilities:
- Multi-provider AI support (Anthropic, OpenAI, Gemini)
- MCP tool execution integration
- Structured output handling
- Sampling and routing logic
- AI request tracking and analytics
- Schema transformation and validation

## Database Usage

This module uses the `DatabaseProvider` abstraction for all database operations.

### Repository Pattern

```rust
use systemprompt_database::DatabaseProvider;
use std::sync::Arc;

pub struct AiRequestRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl AiRequestRepository {
    pub async fn get_request(&self, id: &str) -> Result<Option<AiRequest>> {
        let row = self.db
            .fetch_optional(queries::GET_REQUEST, &[&id])
            .await?;

        row.map(|r| AiRequest::from_json_row(&r)).transpose()
    }
}
```

See [Database Migration Guide](../../../docs/database-migration-guide.md) for details.
