# MCP Server Integration Tests

This directory contains integration tests for MCP servers, organized by server name.

## Structure

```
mcp/
├── content-research/       # Content Research MCP Server
│   ├── list_content.test.ts
│   └── analytics.test.ts
├── tyingshoelaces/         # TyingShoeaces Blog MCP Server
│   ├── introduction.test.ts
│   └── search.test.ts
└── README.md
```

## Test Organization

Each MCP server has its own subdirectory containing:
- One test file per tool
- Tests that connect to the running MCP server
- Tests that call each tool with various parameters
- Tests that print and verify the results

## Running Tests

### Prerequisites

1. Start the API server with MCP servers:
   ```bash
   just start
   ```

2. Generate an admin token:
   ```bash
   export ADMIN_TOKEN=$(just admin-token)
   ```

### Run All MCP Tests

```bash
cd core/tests/integration
npm test mcp/
```

### Run Specific Server Tests

```bash
# Content Research server
npm test mcp/content-research

# TyingShoeaces server
npm test mcp/tyingshoelaces
```

### Run Individual Tool Tests

```bash
npm test mcp/content-research/list_content.test.ts
npm test mcp/tyingshoelaces/search.test.ts
```

## Test Coverage

### content-research Server

**Tools:**
- `list_content` - Browse published content with filtering
  - Test without parameters (default limit)
  - Test with limit parameter
  - Test with category filter
  - Verify table artifact structure

- `content_analytics` - Query performance metrics
  - Test with topic parameter
  - Test with date range and metrics
  - Test missing required parameters

### tyingshoelaces Server

**Tools:**
- `introduction` - Get blog introduction
  - Test tool execution
  - Verify webpage artifact creation
  - Check structured content

- `search` - Search blog content
  - Test with query parameter
  - Test with limit parameter
  - Test missing/empty query handling
  - Verify webpage artifact for top result

## Test Output

Each test prints detailed results:
- ✅ Success/Error status
- 📊 Tool results (tables, dashboards, artifacts)
- 📄 Structured content details
- ❌ Error messages and codes

## Adding New Tests

1. Create a new directory for your MCP server:
   ```bash
   mkdir -p core/tests/integration/mcp/my-server
   ```

2. Create test files for each tool:
   ```typescript
   import { describe, it, expect, beforeAll } from 'vitest';
   import { generateAdminToken } from '@test/utils/auth-utils';
   import config from '@test/config';

   describe('MCP: my-server - tool_name', () => {
     let adminToken: string;
     const serverName = 'my-server';

     beforeAll(() => {
       adminToken = generateAdminToken();
     });

     it('should execute tool', async () => {
       const response = await fetch(
         `${config.apiBaseUrl}/api/v1/mcp/${serverName}/mcp`,
         {
           method: 'POST',
           headers: {
             'Authorization': `Bearer ${adminToken}`,
             'Content-Type': 'application/json',
             'mcp-protocol-version': '2024-11-05',
           },
           body: JSON.stringify({
             jsonrpc: '2.0',
             id: 1,
             method: 'tools/call',
             params: {
               name: 'tool_name',
               arguments: {},
             },
           }),
         }
       );

       expect(response.ok).toBe(true);
       const result = await response.json();

       console.log('Result:', JSON.stringify(result, null, 2));

       expect(result.result).toBeDefined();
     });
   });
   ```

3. Run your tests:
   ```bash
   npm test mcp/my-server
   ```

## Troubleshooting

### Server Not Running
```
Error: fetch failed (connection refused)
```
**Solution:** Start the API server with `just start`

### Missing Token
```
Error: 401 Unauthorized
```
**Solution:** Generate and export admin token:
```bash
export ADMIN_TOKEN=$(just admin-token)
```

### Tool Not Found
```
Error: -32601 Method not found
```
**Solution:** Verify tool name matches the server's tool registration

## Related Documentation

- [MCP Protocol Spec](https://modelcontextprotocol.io/)
- [Vitest Documentation](https://vitest.dev/)
- [Test Setup Guide](/core/tests/integration/README.md)
