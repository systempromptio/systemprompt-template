# SystemPrompt Admin MCP Tests

## Overview

Comprehensive integration tests for the `systemprompt-admin` MCP server, validating all admin tools for system analytics, monitoring, and management.

## Test Suite

### Test Files (7 total)

1. **01-dashboard.test.ts** (6 tests)
   - Dashboard tool with multiple time ranges
   - Real-time, 24h, 7d, 30d metrics
   - Section structure validation

2. **02-users.test.ts** (8 tests)
   - User management operations
   - List users, get user details
   - Update user roles
   - List sessions (active/recent)
   - User activity analysis

3. **03-traffic.test.ts** (9 tests)
   - Traffic analytics with time ranges
   - Device, geo, client breakdowns
   - Landing pages and traffic sources
   - UTM campaign tracking

4. **04-conversations.test.ts** (8 tests)
   - Conversation analytics
   - Agent performance metrics
   - Success rates and message counts
   - Status distribution

5. **05-subjects.test.ts** (11 tests)
   - Subject and topic analysis
   - Quality score tracking
   - User satisfaction metrics
   - Location breakdown
   - Recent evaluations

6. **06-activity-summary.test.ts** (6 tests)
   - Quick activity summary
   - Visitor trends
   - Conversation counts
   - Tool usage metrics

7. **07-content.test.ts** (12 tests) ✨ **NEW**
   - Content performance analytics
   - Top content by views/engagement
   - Category performance
   - Content trends and lifecycle
   - Sort modes validation

## Test Coverage

### Total Tests: **60 tests**
- All passing ✅
- 100% tool coverage
- Multiple parameter combinations tested
- Error handling validated

### Tools Tested

| Tool | Test File | Tests | Status |
|------|-----------|-------|--------|
| dashboard | 01-dashboard.test.ts | 6 | ✅ Pass |
| user | 02-users.test.ts | 8 | ✅ Pass |
| traffic | 03-traffic.test.ts | 9 | ✅ Pass |
| conversations | 04-conversations.test.ts | 8 | ✅ Pass |
| subjects | 05-subjects.test.ts | 11 | ✅ Pass |
| activity_summary | 06-activity-summary.test.ts | 6 | ✅ Pass |
| content | 07-content.test.ts | 12 | ✅ Pass |

## Running Tests

### All Admin Tests

```bash
cd core/tests/integration
npx vitest run mcp/systemprompt-admin
```

### Specific Test File

```bash
npx vitest run mcp/systemprompt-admin/07-content.test.ts
```

### Watch Mode

```bash
npx vitest mcp/systemprompt-admin
```

### With Verbose Output

```bash
npx vitest run mcp/systemprompt-admin --reporter=verbose
```

## Prerequisites

1. **API Server Running**
   ```bash
   cd /var/www/html/systemprompt-blog
   ./core/target/debug/systemprompt serve api
   ```

2. **Database Initialized**
   ```bash
   ./core/target/debug/systemprompt db migrate
   ```

3. **MCP Servers Enabled**
   - systemprompt-admin must be in config and enabled

## Test Structure

Each test file follows this pattern:

```typescript
describe('MCP: systemprompt-admin - {tool} tool', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;

  beforeAll(async () => {
    // Setup admin authentication
    adminToken = generateAdminToken();
    contextId = await createContext(adminToken, config.apiBaseUrl);

    // Connect to MCP server
    client = new McpTestClient({...});
    await client.connect();
  });

  afterAll(async () => {
    await client.close();
  });

  it('should list tools and find {tool}', async () => {
    // Verify tool exists in tool list
  });

  it('should execute {tool} with various parameters', async () => {
    // Test tool execution with different inputs
  });

  it('should validate output structure', async () => {
    // Verify response format and data structure
  });
});
```

## Recent Changes

### 2024-11-10: Content Tool Tests Added

**Issue**: Content tool had SQL errors that weren't caught by tests
- Problem: `engagement_score` column alias referenced in ORDER BY CASE
- Root cause: No test coverage for content tool

**Fix**:
1. ✅ Fixed SQL queries (using CTEs)
   - `/core/crates/modules/blog/src/queries/core/analytics/get_top_content.sql`
   - `/core/crates/modules/blog/src/queries/core/analytics/get_category_performance.sql`

2. ✅ Created comprehensive test file
   - `07-content.test.ts` with 12 tests
   - Tests all time ranges (7d, 30d, 90d)
   - Tests both sort modes (views, engagement)
   - Validates structure and sections

3. ✅ All tests passing
   - Test suite: 60/60 tests ✅
   - 100% tool coverage achieved

## Test Data Notes

### Expected Behaviors

**No Data Scenarios** (common in test environment):
- `content`: Returns empty sections if no `content_view_events` exist
- `subjects`: Shows "No Evaluation Data" if evaluation system not running
- `traffic`: May have minimal data if analytics events are sparse

**Data Requirements**:
- **content**: Requires `content_view_events` table with view data
- **subjects**: Requires conversation evaluation system running
- **traffic**: Requires `analytics_events` and `user_sessions` data
- **conversations**: Requires `user_contexts` with conversation data
- **users**: Requires user accounts in database

### Sample Output

When data is available, tests show detailed metrics:

```
📊 Content Dashboard:
  Title: Content Performance Analytics
  Description: Content metrics for the last 30 days
  Sections: 3

  📍 Section: Top Performing Content
     Type: list
     Items: 15 articles

  📍 Section: Category Performance
     Type: list
     Categories: 5

  📍 Section: Content Trends & Lifecycle
     Type: list
     Trend items: 15
```

## Debugging Failed Tests

### Check API Server

```bash
curl http://localhost:8080/api/v1/health
```

### Check MCP Server

```bash
curl http://localhost:8080/api/v1/mcp/servers
```

### View Logs

```bash
tail -f /tmp/api.log
```

### Database Query Test

```bash
./core/target/debug/systemprompt db query "SELECT COUNT(*) FROM content_view_events"
```

## Success Criteria

✅ All 60 tests passing
✅ All 7 tools tested
✅ Multiple parameter combinations validated
✅ Error scenarios handled gracefully
✅ Data structure validation complete
✅ Admin authentication working

## Contributing

When adding new admin tools:

1. **Create Test File**
   - Follow naming: `0X-toolname.test.ts`
   - Copy structure from existing tests

2. **Test Coverage**
   - Tool discovery (list tools)
   - Default parameters
   - All parameter combinations
   - Time ranges if applicable
   - Output structure validation
   - Section/data validation

3. **Run Tests**
   - Ensure all existing tests still pass
   - New tests should cover edge cases
   - Document any data requirements

4. **Update This README**
   - Add to test files list
   - Update total test count
   - Note any special requirements

---

**Last Updated**: 2024-11-10
**Test Files**: 7
**Total Tests**: 60
**Status**: All Passing ✅
