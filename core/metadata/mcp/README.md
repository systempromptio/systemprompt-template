# MCP Server Registry

This directory contains metadata for registered MCP (Model Context Protocol) servers.

## Structure

```
metadata/mcp/
├── README.md                    # This file
└── <server-name>.json          # MCP server metadata
```

## Metadata Format

Each MCP server has a JSON metadata file describing its capabilities:

```json
{
  "name": "server-name",
  "displayName": "Human Readable Name",
  "description": "What this MCP server does",
  "version": "1.0.0",
  "transport": "stdio",
  "command": "cargo",
  "args": ["run", "--bin", "server-name"],
  "env": {
    "ENV_VAR": "value"
  },
  "tools": [
    {
      "name": "tool-name",
      "description": "What this tool does",
      "inputSchema": {
        "type": "object",
        "properties": {
          "param": {
            "type": "string",
            "description": "Parameter description"
          }
        },
        "required": ["param"]
      }
    }
  ],
  "prompts": [
    {
      "name": "prompt-name",
      "description": "What this prompt does",
      "arguments": [
        {
          "name": "arg",
          "description": "Argument description",
          "required": true
        }
      ]
    }
  ],
  "resources": [
    {
      "uri": "resource://path",
      "name": "Resource Name",
      "description": "What this resource provides",
      "mimeType": "text/plain"
    }
  ]
}
```

## Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique server identifier |
| `displayName` | string | Human-readable name |
| `description` | string | Purpose of the server |
| `version` | string | Semantic version |
| `transport` | string | `"stdio"` or `"http"` |
| `command` | string | Executable command |
| `args` | array | Command arguments |

## Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `env` | object | Environment variables |
| `tools` | array | Available tools |
| `prompts` | array | Available prompts |
| `resources` | array | Available resources |

## Transport Types

### stdio (Local Process)

Standard input/output communication:

```json
{
  "transport": "stdio",
  "command": "cargo",
  "args": ["run", "--bin", "my-server"]
}
```

### http (Remote Service)

HTTP-based communication:

```json
{
  "transport": "http",
  "url": "http://localhost:8080/mcp",
  "headers": {
    "Authorization": "Bearer ${MCP_TOKEN}"
  }
}
```

## Tool Schema

Tools define their input schema using JSON Schema:

```json
{
  "name": "create-user",
  "description": "Create a new user account",
  "inputSchema": {
    "type": "object",
    "properties": {
      "username": {
        "type": "string",
        "description": "User's username",
        "minLength": 3,
        "maxLength": 20
      },
      "email": {
        "type": "string",
        "description": "User's email address",
        "format": "email"
      },
      "role": {
        "type": "string",
        "description": "User role",
        "enum": ["user", "admin"]
      }
    },
    "required": ["username", "email"]
  }
}
```

## Prompt Schema

Prompts can have arguments:

```json
{
  "name": "code-review",
  "description": "Review code for quality issues",
  "arguments": [
    {
      "name": "language",
      "description": "Programming language",
      "required": true
    },
    {
      "name": "strictness",
      "description": "Review strictness level",
      "required": false
    }
  ]
}
```

## Resource Schema

Resources expose data or files:

```json
{
  "uri": "resource://docs/api",
  "name": "API Documentation",
  "description": "Complete API reference",
  "mimeType": "text/markdown"
}
```

## Registration Process

### 1. Create Metadata File

```bash
cat > metadata/mcp/my-server.json <<EOF
{
  "name": "my-server",
  "displayName": "My MCP Server",
  "description": "Custom MCP server for X",
  "version": "1.0.0",
  "transport": "stdio",
  "command": "cargo",
  "args": ["run", "--bin", "my-server"],
  "tools": []
}
EOF
```

### 2. Validate Metadata

```bash
# Start server to validate metadata
just mcp start my-server

# Check status
just mcp status
```

### 3. Test Tools

```bash
# List available tools
just mcp exec my-server --list-tools

# Execute a tool
just mcp exec my-server tool-name '{"param": "value"}'
```

## Environment Variables

Use environment variable substitution in metadata:

```json
{
  "env": {
    "API_KEY": "${MY_API_KEY}",
    "DATABASE_URL": "${DATABASE_URL}",
    "LOG_LEVEL": "info"
  }
}
```

Variables are resolved from:
1. Shell environment
2. `.env` file
3. SystemPrompt config

## Best Practices

1. **Semantic versioning** - Use semver for versions
2. **Clear descriptions** - Explain what each tool does
3. **Complete schemas** - Include all parameters with descriptions
4. **Environment variables** - Use for secrets and config
5. **Validation** - Test metadata before committing
6. **Documentation** - Update README when adding servers

## Example: Filesystem MCP Server

```json
{
  "name": "filesystem",
  "displayName": "Filesystem MCP Server",
  "description": "Read and write files on the local filesystem",
  "version": "1.0.0",
  "transport": "stdio",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-filesystem", "/app"],
  "env": {},
  "tools": [
    {
      "name": "read_file",
      "description": "Read contents of a file",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "File path to read"
          }
        },
        "required": ["path"]
      }
    },
    {
      "name": "write_file",
      "description": "Write contents to a file",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "File path to write"
          },
          "content": {
            "type": "string",
            "description": "Content to write"
          }
        },
        "required": ["path", "content"]
      }
    }
  ]
}
```

## Troubleshooting

**Server not found:**
```bash
# Check metadata file exists
ls metadata/mcp/my-server.json

# Verify JSON syntax
cat metadata/mcp/my-server.json | jq .
```

**Invalid metadata:**
```bash
# View error details
just log

# Common issues:
# - Missing required fields
# - Invalid JSON syntax
# - Wrong transport type
# - Invalid schema format
```

**Command not found:**
```bash
# Test command manually
cargo run --bin my-server

# Check binary exists
ls target/debug/my-server
```

## See Also

- `/templates/mcp/` - MCP server templates
- `/templates/agents/` - Agent templates
- `CLAUDE.md` - Service architecture
- [MCP Specification](https://spec.modelcontextprotocol.io/)
