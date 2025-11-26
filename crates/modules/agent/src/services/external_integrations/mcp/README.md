# MCP Integration Module

This module handles dynamic skill loading from MCP (Model Context Protocol) servers for A2A (Agent-to-Agent) agents.

## Architecture Overview

**Key principle**: Skills are NOT persisted in the database. They are loaded dynamically on-demand from running MCP services.

### Flow Diagram

```
1. Agent saves server IDs in metadata (JSON array)
   ↓
2. We load array of servers from agent_metadata.mcp_servers
   ↓
3. Compare with services table (to check MCP server state)
   ↓
4. Use service state (contains port/host) to load MCP client
   ↓
5. Use rmcp.listTool() to fetch tools from running servers
   ↓
6. Convert MCP tools into A2A skills using SkillConverter
   ↓
7. Return skills and server information
```

## Components

### 1. McpSkillLoader (skill_loader.rs)
**Main orchestrator** that coordinates the entire skill loading process:
- `load_agent_skills(agent_id)` - Load all skills for an agent
- `load_server_skills(server_name)` - Load skills from specific server
- `get_all_agent_skills_map()` - Get skills for all agents (used by task processor)

### 2. ServiceStateManager (service/state_manager.rs)
**Queries the services table** to get MCP server connection information:
- `get_mcp_service(name)` - Get connection info for specific MCP server
- `list_mcp_services()` - List all MCP services
- `list_running_mcp_services()` - List only running MCP services

### 3. McpClientAdapter (client/adapter.rs)
**Connects to MCP servers** and fetches tools:
- `fetch_tools(host, port)` - Connect and get tools from MCP server
- `fetch_tools_with_timeout()` - Same with timeout protection
- `test_connection()` - Test if server is reachable

### 4. SkillConverter (converter/tool_to_skill.rs)
**Converts MCP tools to A2A skills**:
- `convert_tool()` - Convert single MCP tool to AgentSkill
- `convert_tools()` - Convert multiple tools
- `convert_tools_with_aggregate()` - Convert + create aggregate server skill

## Data Flow

### Storage Schema

```sql
-- Agent metadata stores MCP server assignments
agent_metadata.mcp_servers: JSON array ["database-tools", "file-manager"]

-- Services table stores running MCP server state
services: {
  name: "database-tools",
  host: "localhost",
  port: 5001,
  protocol: "mcp",
  status: "running"
}
```

### Runtime Process

1. **Agent Assignment**: Agent metadata contains `mcp_servers: ["database-tools", "file-manager"]`

2. **Service Lookup**: For each server name:
   ```sql
   SELECT host, port, status FROM services
   WHERE protocol = 'mcp' AND name = 'database-tools'
   ```

3. **MCP Connection**: Connect to `localhost:5001` using rmcp protocol

4. **Tool Fetching**: Call `McpClient::list_tools()` to get available tools

5. **Skill Conversion**: Transform MCP tools into AgentSkill objects:
   ```rust
   AgentSkill {
     id: "database-tools/query_table",
     name: "query_table",
     description: "Query database table",
     tags: ["database-tools", "mcp-tool"],
     // ...
   }
   ```

6. **Return**: Combined skills from all assigned MCP servers

## Key Features

- **Dynamic Loading**: No skill persistence - always fresh from MCP servers
- **Timeout Protection**: 5-second timeout prevents hanging on dead servers
- **Error Resilience**: Failed servers don't block other server loading
- **Aggregate Skills**: Creates server-level skills for multi-tool servers
- **Performance Monitoring**: Timing and logging for debugging

## Usage Examples

```rust
// Basic usage
let loader = McpSkillLoader::new(db_pool);
let skills = loader.load_agent_skills("echo-agent").await?;

// Check specific server
let server_skills = loader.load_server_skills("database-tools").await?;

// For task processor
let all_skills = loader.get_all_agent_skills_map().await?;
```

## Integration Points

- **CLI Commands**: `systemprompt-a2a mcp list echo-agent`
- **Task Processor**: Uses `get_all_agent_skills_map()` for capability matching
- **Agent Registry**: Skills loaded during agent listing/display
- **Services Table**: Depends on MCP services being registered and running

## Error Handling

- Server not found → Skip with warning
- Server not running → Skip with warning
- Connection timeout → Skip with warning
- No tools returned → Empty skill list (not error)
- JSON parse errors → Propagated as errors

This ensures the system is resilient and continues operating even when some MCP servers are unavailable.