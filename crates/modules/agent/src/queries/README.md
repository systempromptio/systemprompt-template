# A2A Queries Directory

This directory contains all SQL queries for the A2A (Agent-to-Agent) communication crate. It follows a strict architectural pattern where SQL files are the single source of truth for database operations.

## Architecture Principles

### 1. Repository Pattern
All database access **MUST** go through the repository layer:
```
Service Layer → Repository Layer → Database
     ↓               ↓
No SQL here    All SQL execution here
```

### 2. Single Source of Truth
- SQL files in this directory are the **only** place SQL is defined
- Repositories use `include_str!()` to load SQL content
- No SQL should be written inline in Rust code

### 3. No Direct Query Access
**❌ WRONG - Direct query access outside repository:**
```rust
// In service or handler - DON'T DO THIS
let agents = sqlx::query("SELECT * FROM agents").fetch_all(&pool).await?;
```

**✅ CORRECT - Access through repository:**
```rust
// In service layer
let agents = agent_repository.list_active().await?;

// In repository implementation
pub async fn list_active(&self) -> Result<Vec<Agent>> {
    const QUERY: &str = include_str!("../queries/agents/list_active.sql");
    sqlx::query_as(QUERY).fetch_all(&self.pool).await
}
```

## Directory Structure

```
queries/
├── core/           # Core A2A operations
│   ├── agents/     # Agent management queries
│   ├── messages/   # Message routing and storage
│   ├── sessions/   # Communication session management
│   └── protocols/  # Protocol version and negotiation
├── migrations/     # Schema and data migrations
├── fixtures/       # Test data (all test queries here)
│   ├── agents.sql
│   ├── messages.sql
│   └── sessions.sql
└── README.md
```

## A2A-Specific Query Categories

### Agent Lifecycle Queries
```
agents/
├── create_agent.sql          # Register new agent
├── update_agent_status.sql   # Online/offline status
├── find_agents_by_capability.sql  # Discovery by capability
├── heartbeat_update.sql      # Keep-alive mechanism
└── cleanup_inactive.sql      # Remove stale agents
```

### Message Routing Queries
```
messages/
├── send_message.sql          # Store outbound message
├── receive_message.sql       # Mark message as received  
├── route_to_agent.sql        # Find target agent for message
├── get_pending_messages.sql  # Retrieve queued messages
└── archive_old_messages.sql  # Cleanup processed messages
```

### Session Management Queries
```
sessions/
├── create_session.sql        # Start new communication session
├── join_session.sql          # Add agent to existing session
├── update_session_state.sql  # Track session progress
├── close_session.sql         # End communication session
└── find_sessions_by_agent.sql # Agent's active sessions
```

### Protocol Negotiation Queries
```
protocols/
├── get_supported_versions.sql # Agent protocol capabilities
├── negotiate_protocol.sql     # Find common protocol version
├── update_protocol_support.sql # Update agent capabilities
└── get_protocol_config.sql    # Protocol-specific settings
```

## Best Practices for A2A Queries

### 1. Agent-Centric Design
Queries should support the distributed nature of A2A communication:

```sql
-- find_available_agents.sql
-- Find agents that can handle a specific message type
SELECT a.id, a.name, a.endpoint, a.capabilities
FROM agents a
WHERE a.status = 'online'
  AND a.last_heartbeat > datetime('now', '-30 seconds')
  AND json_extract(a.capabilities, '$.message_types') LIKE '%' || ? || '%'
ORDER BY a.load_factor ASC
LIMIT ?
```

### 2. Message Ordering and Delivery
Ensure reliable message delivery with proper ordering:

```sql
-- get_pending_messages.sql
-- Retrieve messages in order with retry logic
SELECT m.id, m.from_agent, m.to_agent, m.content, m.message_type,
       m.created_at, m.retry_count
FROM messages m
WHERE m.to_agent = ?
  AND m.status = 'pending'
  AND (m.retry_after IS NULL OR m.retry_after <= datetime('now'))
ORDER BY m.priority DESC, m.created_at ASC
LIMIT ?
```

### 3. Session State Management
Track multi-agent communication sessions:

```sql
-- update_session_participants.sql
-- Add/remove agents from communication session
INSERT OR REPLACE INTO session_participants (session_id, agent_id, role, joined_at)
VALUES (?, ?, ?, datetime('now'))
```

### 4. Performance and Cleanup
A2A systems generate high message volume, requiring efficient cleanup:

```sql
-- cleanup_processed_messages.sql  
-- Remove old processed messages to prevent database bloat
DELETE FROM messages 
WHERE status IN ('delivered', 'failed')
  AND created_at < datetime('now', '-7 days')
  AND retry_count >= 3
```

## File Organization Examples

### Core Operational Queries
```sql
-- agents/register_agent.sql
-- Register new agent with capabilities
INSERT INTO agents (id, name, endpoint, capabilities, status, created_at)
VALUES (?, ?, ?, ?, 'online', datetime('now'))
ON CONFLICT (id) DO UPDATE SET
  endpoint = excluded.endpoint,
  capabilities = excluded.capabilities,
  status = excluded.status,
  last_heartbeat = datetime('now')
RETURNING *
```

### Message Processing Queries
```sql
-- messages/route_message.sql
-- Find best agent to handle message based on capabilities and load
WITH available_agents AS (
  SELECT a.id, a.endpoint, a.load_factor,
         json_extract(a.capabilities, '$.max_concurrent') as max_concurrent,
         COUNT(m.id) as current_messages
  FROM agents a
  LEFT JOIN messages m ON a.id = m.to_agent AND m.status = 'processing'
  WHERE a.status = 'online'
    AND json_extract(a.capabilities, '$.message_types') LIKE '%' || ? || '%'
  GROUP BY a.id
)
SELECT id, endpoint
FROM available_agents
WHERE current_messages < max_concurrent
ORDER BY load_factor ASC, current_messages ASC
LIMIT 1
```

### Session Coordination Queries
```sql
-- sessions/create_multi_agent_session.sql
-- Start new session with multiple participants
INSERT INTO sessions (id, initiator_agent, session_type, protocol_version, state, created_at)
VALUES (?, ?, ?, ?, 'initializing', datetime('now'))
RETURNING *
```

## Repository Integration

### Agent Repository Example
```rust
pub struct AgentRepository {
    pool: SqlitePool,
}

impl AgentRepository {
    const SAVE_AGENT_CARD: &'static str = include_str!("../queries/core/agents/save_agent_card.sql");
    const GET_AGENT_CARD: &'static str = include_str!("../queries/core/agents/get_agent_card.sql");
    const LIST_AGENT_CARDS: &'static str = include_str!("../queries/core/agents/list_agent_cards.sql");
    
    pub async fn save_agent_card(&self, agent_id: &str, card: &AgentCard) -> Result<()> {
        let card_json = serde_json::to_string(card)?;
        sqlx::query(Self::SAVE_AGENT_CARD)
            .bind(agent_id)
            .bind(&card.name)
            .bind(&card.description)
            .bind(&card.version)
            .bind(&card_json)
            .execute(&self.pool)
            .await
            .map_err(Into::into)
    }
    
    pub async fn get_agent_card(&self, agent_id: &str) -> Result<Option<AgentCard>> {
        let row = sqlx::query(Self::GET_AGENT_CARD)
            .bind(agent_id)
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
            let card_json: String = row.get("agent_card_json");
            let card: AgentCard = serde_json::from_str(&card_json)?;
            Ok(Some(card))
        } else {
            Ok(None)
        }
    }
}
```

## A2A-Specific Considerations

### 1. Distributed System Queries
Handle eventual consistency and network partitions:
- Use timestamps for ordering
- Include retry logic in queries
- Support idempotent operations

### 2. Real-time Communication
Optimize for low-latency message routing:
- Index on agent status and capabilities
- Pre-compute routing tables where possible
- Use efficient JSON queries for capability matching

### 3. Scalability
Design queries to handle high agent/message volume:
- Partition messages by time or agent
- Use batch operations for bulk updates
- Implement efficient cleanup procedures

### 4. Protocol Evolution  
Support multiple protocol versions:
- Store protocol version with each agent
- Include version compatibility in routing queries
- Handle protocol negotiation in session queries

## Anti-Patterns to Avoid

❌ **Direct SQL in services** - Breaks repository pattern
❌ **Inline JSON parsing** - Use database JSON functions
❌ **Missing cleanup queries** - Causes database bloat  
❌ **No retry logic** - Leads to message loss
❌ **Hardcoded timeouts** - Makes system inflexible

## Testing A2A Queries

### Multi-Agent Test Scenarios
```rust
#[tokio::test] 
async fn test_message_routing_with_load_balancing() {
    let pool = setup_test_db().await;
    let repo = MessageRepository::new(pool.clone());
    
    // Create agents with different load factors
    create_test_agents(&pool, &[
        ("agent1", 0.1), // Low load
        ("agent2", 0.8), // High load  
        ("agent3", 0.3), // Medium load
    ]).await;
    
    // Route message should prefer agent1 (lowest load)
    let target = repo.route_message("test_capability").await.unwrap();
    assert_eq!(target.agent_id, "agent1");
}
```

The A2A queries directory is critical for reliable distributed agent communication. All queries should be designed with the distributed, real-time nature of agent-to-agent systems in mind.