# MCP Management

Enterprise Model Context Protocol (MCP) server management for governing tool access, execution logging, and agent-tool relationships at scale.

## MCP Server Registry

### Listing and Managing Servers

```bash
# List all registered MCP servers
systemprompt plugins mcp list

# Show MCP server details (port, OAuth, tools)
systemprompt plugins mcp show <server-name>

# View MCP server logs
systemprompt plugins mcp logs <server-name>

# Follow MCP logs in real-time
systemprompt plugins mcp logs <server-name> -f
```

### Port Management

MCP servers are assigned to port range 5000-5999:
- Each server gets a dedicated port
- Port conflicts are detected at startup
- Health checks verify server availability

### OAuth2 Per Server

Each MCP server has its own OAuth2 configuration:
- Configurable scopes (admin, mcp, a2a)
- Scope-based access control per tool
- Token validation on every tool call

## Tool Governance

### Tool Execution Tracking

Every MCP tool call is logged:

```bash
# List all tool executions
systemprompt infra logs tools list

# Filter by error status
systemprompt infra logs tools list --status error

# Filter by server
systemprompt infra logs tools list --server <server-name>

# Tool usage analytics
systemprompt analytics tools stats

# Deep dive on specific tool
systemprompt analytics tools show <tool-name>
```

### Agent-Tool Mapping

Control which agents can access which tools:

```bash
# View tools available to an agent
systemprompt admin agents tools <agent-name>

# View detailed tool descriptions
systemprompt admin agents tools <agent-name> --detailed
```

Agent-tool access is governed through:
1. **MCP server assignment**: Agents are linked to specific MCP servers in their config
2. **OAuth scopes**: Server-level scopes control access
3. **Plugin boundaries**: Plugins define which MCP servers are available

## MCP Configuration

### AI Provider Configuration

```yaml
mcp:
  auto_discover: true          # Automatic MCP server discovery
  connect_timeout_ms: 5000     # Connection timeout
  execution_timeout_ms: 30000  # Tool execution timeout
  retry_attempts: 3            # Retry on failure
```

### Server Types

| Server Type | Purpose | Typical Scope |
|------------|---------|---------------|
| CLI Execution | Execute platform CLI commands | admin |
| Marketplace | Skill and agent management | mcp |
| Skill Manager | Cloud-based skill sync | mcp |
| Custom | Domain-specific tools | configurable |

## Integration Patterns

### MCP for Agent Interoperability

MCP enables standardized agent-to-tool communication:

1. **Tool Discovery**: Agents discover available tools through MCP server registry
2. **Capability Negotiation**: Agents query tool schemas before invocation
3. **Secure Invocation**: OAuth2 tokens validated on every tool call
4. **Result Streaming**: Support for streaming responses from long-running tools

### Agent-to-Agent via MCP

When agents need to communicate:

1. Agent A sends a message via the A2A protocol
2. The receiving agent's MCP tools are available for task execution
3. All inter-agent communication is logged and auditable
4. Timeouts and retries are configurable per interaction

### Scale Considerations

For enterprise-scale deployments:
- MCP rate limiting at 200 req/s (5x multiplier for service accounts)
- Connection pooling for MCP servers
- Auto-discovery reduces manual configuration
- Centralized logging for all tool executions across all servers
