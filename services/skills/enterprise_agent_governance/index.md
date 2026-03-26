# Agent Governance

Enterprise-scale agent governance for managing AI agent fleets. This skill covers agent lifecycle management, policy enforcement, access control, and behavioral monitoring.

## Agent Fleet Management

### Listing and Monitoring Agents

```bash
# List all registered agents with their status
systemprompt admin agents list

# Check running agents and health
systemprompt admin agents status

# View agent registry from the gateway
systemprompt admin agents registry --running

# View detailed agent configuration
systemprompt admin agents show <agent-name>
```

### Agent Lifecycle

```bash
# Create a new agent with specific port and configuration
systemprompt admin agents create --name <name> --port <port>

# Enable/disable agents
systemprompt admin agents edit <name> --enable
systemprompt admin agents edit <name> --disable

# Validate agent configuration before deployment
systemprompt admin agents validate <name>

# Delete an agent (requires confirmation)
systemprompt admin agents delete <name> -y
```

### Agent-to-Agent Communication

```bash
# Send async message to an agent
systemprompt admin agents message <name> -m "perform security scan"

# Send blocking message and wait for response
systemprompt admin agents message <name> -m "analyze this data" --blocking --timeout 60

# Stream response in real-time
systemprompt admin agents message <name> -m "generate report" --stream

# Check task status
systemprompt admin agents task <name> --task-id <id>
```

## Access Control Policies

### Role-Based Agent Access

Agents are governed by role-based access control (RBAC):

- **Admin agents**: Full platform access, restricted to admin role
- **User agents**: Standard operations, available to all authenticated users
- **Service agents**: Internal orchestration with elevated rate limits
- **A2A agents**: Agent-to-agent communication with dedicated scopes

### Plugin-Based Governance

Agents are organized into plugins (governed bundles):

```bash
# List all plugins and their agent assignments
systemprompt core plugins list

# View plugin detail including agents, skills, and MCP servers
systemprompt core plugins show <plugin-id>
```

Each plugin defines:
- Which agents are included
- Which skills those agents can access
- Which MCP servers (tools) they can use
- Required roles for access
- Plugin dependencies

## Behavioral Monitoring

### Agent Logging and Tracing

```bash
# View agent-specific logs
systemprompt admin agents logs <name>

# Follow logs in real-time
systemprompt admin agents logs <name> -f

# List execution traces for an agent
systemprompt infra logs trace list --agent <name>

# View failed traces only
systemprompt infra logs trace list --agent <name> --status failed

# Full trace details including MCP tool calls
systemprompt infra logs trace show <trace-id> --all
```

### Agent Performance Analytics

```bash
# Agent performance rankings
systemprompt analytics agents stats

# Deep dive on specific agent
systemprompt analytics agents show <agent-name>

# Sort agents by cost, usage, or error rate
systemprompt analytics agents list --sort-by cost
```

## Governance Patterns

### Super-Agent Architecture

Organize agents into hierarchical super-agents:

1. **Primary Agent**: Customer-facing, handles initial request routing
2. **Domain Agents**: Specialized agents for specific business functions
3. **Service Agents**: Internal agents for cross-cutting concerns
4. **Admin Agent**: Platform governance and monitoring

### Agent Consolidation

Reduce agent sprawl by:
- Grouping related agents into plugins
- Sharing skills across agents via skill composition
- Using MCP tools for standardized capability exposure
- Enforcing naming conventions and documentation standards

### Scale Considerations (enterprise-scale deployments)

- **Port Management**: Agents assigned to port range 9000-9999
- **Rate Limiting**: Tiered limits per authentication scope
- **Stateless Design**: Agents can be horizontally scaled
- **Health Checks**: Automated health monitoring for all agents
