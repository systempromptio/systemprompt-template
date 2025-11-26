# Hooks Documentation

Comprehensive guide to the application's React hooks ecosystem.

## Quick Start

Import hooks from the central barrel export:

```typescript
import { useA2AClient, useMcpRegistry, useContextStream } from '@/hooks'
```

## Hook Categories

### 🔐 Authentication & Token Management

- **useAuth** - Authentication state and methods
- **useTokenExpiryMonitor** - Monitors and refreshes tokens automatically
- **useFirstVisit** - Detects first-time visitors

### 🤖 Agent & MCP Management

- **useAgentDiscovery** - Discovers and registers agents
- **useA2AClient** - Manages A2A client connections to agents
- **useMcpRegistry** - Loads MCP servers and tools
- **useMcpToolCaller** - Executes MCP tools with parameter handling
- **useToolParameters** - Manages tool parameter collection

### 📡 Context & Streaming

- **useContextStream** - Coordinates real-time context updates (refactored)
- **useSSEConnection** - Manages SSE connection lifecycle
- **useStreamEventProcessor** - Routes and processes SSE events
- **useContextInit** - Initializes default conversation context
- **useArtifactSubscription** - Subscribes to artifact updates

### 📊 Data & Options

- **useDynamicOptions** - Fetches dynamic dropdown options
- **useResolvedSchema** - Resolves form schemas from tool definitions

### 📈 Tracking & Analytics

- **usePageView** - Tracks page view analytics
- **useShareTracking** - Tracks content sharing

### 🛠️ Utilities

- **useRetry** - Manages retry logic with exponential backoff
- **useAsyncState** - Simplifies async operation state management

## Architecture Improvements

### Refactored useContextStream (v2)

**Before**: Monolithic 528-line hook with 10+ responsibilities
**After**: Composed hooks with single responsibilities

```
useContextStream (63 lines - coordinator)
├── useSSEConnection (150 lines - connection mgmt)
├── useStreamEventProcessor (180 lines - event routing)
└── streamEventHandlers (200 lines - event handlers)
```

**Benefits**:
- 88% reduction in main hook complexity
- Each hook is testable in isolation
- Clear separation of concerns
- Easier to debug and maintain

### New Utilities

#### useRetry Hook
Eliminates duplicate retry logic across multiple hooks:

```typescript
const { retry, attempt, isRetrying } = useRetry(
  connectToServer,
  { maxAttempts: 5, initialDelay: 2000, backoff: 'exponential' }
)

await retry()
```

#### useAsyncState Hook
Simplifies async operation state management:

```typescript
const { execute, status, data, error } = useAsyncState(
  async () => {
    const res = await fetch('/api/data')
    return res.json()
  }
)

// Status: 'idle' | 'loading' | 'success' | 'error'
```

#### Timing Constants
Centralized timing configuration:

```typescript
import { TIMING } from '@/constants/timing'

setTimeout(() => ..., TIMING.INPUT_DEBOUNCE) // 300ms
```

## Best Practices

### ✅ DO

1. **Use barrel exports** for cleaner imports:
   ```typescript
   import { useA2AClient } from '@/hooks'
   ```

2. **Handle loading and error states** in components:
   ```typescript
   const { loading, error, data } = hook()

   if (loading) return <Spinner />
   if (error) return <Error message={error.message} />
   return <Content data={data} />
   ```

3. **Use useRetry** for retry logic:
   ```typescript
   const { retry, isRetrying, attempt } = useRetry(asyncFn)
   ```

4. **Check dependencies** in useCallback/useEffect:
   ```typescript
   const handler = useCallback(() => {
     // implementation
   }, [deps])
   ```

5. **Clean up resources** in useEffect:
   ```typescript
   useEffect(() => {
     // setup
     return () => {
       // cleanup - always clear timers, listeners, etc.
     }
   }, [deps])
   ```

### ❌ DON'T

1. **Don't use inline functions** in dependencies:
   ```typescript
   // ❌ BAD
   useEffect(() => { ... }, [() => doSomething()])

   // ✅ GOOD
   const handler = useCallback(() => doSomething(), [])
   useEffect(() => { ... }, [handler])
   ```

2. **Don't ignore ESLint warnings** about dependencies:
   ```typescript
   // ❌ BAD - Hides real issues
   useEffect(() => {
     // Uses `value` but not in deps
   }, []) // eslint-disable-next-line

   // ✅ GOOD - Include all dependencies
   useEffect(() => {
     // ...
   }, [value])
   ```

3. **Don't log sensitive data**:
   ```typescript
   // ❌ BAD
   console.log('User data:', toolParameters)

   // ✅ GOOD - Only log counts/metadata
   logger.debug('Calling tool', {
     paramCount: Object.keys(toolParameters).length
   })
   ```

4. **Don't create new objects in dependency arrays**:
   ```typescript
   // ❌ BAD
   useEffect(() => { ... }, [{ a: 1, b: 2 }])

   // ✅ GOOD - Use useMemo or move outside
   const config = useMemo(() => ({ a: 1, b: 2 }), [])
   useEffect(() => { ... }, [config])
   ```

## Common Patterns

### Authentication Flow
```typescript
const { isAuthenticated } = useAuthStore()
const { client, loading, error } = useA2AClient()

useEffect(() => {
  if (!isAuthenticated) return
  // client auto-initializes when auth is ready
}, [isAuthenticated])
```

### Loading MCP Tools
```typescript
useMcpRegistry() // Loads on mount

const { tools, loading, error } = useToolsStore()

if (loading) return <Spinner />
if (error) return <Error />
return <ToolList tools={tools} />
```

### Executing Tools
```typescript
const { callTool, loading, error } = useMcpToolCaller()

const handleExecute = async (tool, params) => {
  try {
    await callTool(tool.endpoint, tool.name, params)
  } catch (err) {
    setError(err.message)
  }
}
```

### Streaming Context Updates
```typescript
useContextStream() // Auto-connects when auth ready

const { conversations } = useContextStore()

useEffect(() => {
  // React to new conversations
}, [conversations])
```

## Testing Hooks

### Unit Testing
```typescript
import { renderHook, act } from '@testing-library/react'
import { useAsyncState } from '@/hooks'

test('useAsyncState handles success', async () => {
  const { result } = renderHook(() =>
    useAsyncState(async () => ({ data: 'test' }))
  )

  await act(async () => {
    await result.current.execute()
  })

  expect(result.current.status).toBe('success')
  expect(result.current.data).toEqual({ data: 'test' })
})
```

### Testing with React StrictMode
Always test hooks with StrictMode enabled to catch missing cleanups:

```typescript
<React.StrictMode>
  <App />
</React.StrictMode>
```

## Debugging

### Enable Logging
Set environment variable to see hook debug logs:

```bash
VITE_LOG_LEVEL=debug npm run dev
```

### Common Issues

**Issue**: Multiple reconnection attempts
- **Cause**: Missing dependency in useEffect
- **Fix**: Ensure all dependencies are included

**Issue**: Memory leaks
- **Cause**: Missing cleanup in useEffect
- **Fix**: Always return cleanup function for timers/listeners

**Issue**: Stale data
- **Cause**: Dependency array issues
- **Fix**: Use ESLint plugin for exhaustive deps

## Migration Guide

### From useContextStream (old) to (new)

No changes needed! External API is identical.

```typescript
// Same API as before
const { connect, disconnect } = useContextStream()
```

Internal implementation is now split across specialized hooks, but consumers see no difference.

## Resources

- [React Hooks Documentation](https://react.dev/reference/react/hooks)
- [Hooks Rules](https://react.dev/warnings/invalid-hook-call-warning)
- [TypeScript Hook Patterns](https://www.typescriptlang.org/docs/handbook/2/narrowing.html)
- [Testing Library Hooks](https://testing-library.com/docs/react-testing-library/example-intro)

## Contributing

When adding new hooks:

1. **Follow naming convention**: `use[Feature]` (e.g., `useAsyncState`)
2. **Add JSDoc comments**: Explain what hook does
3. **Include examples**: Show basic usage in JSDoc
4. **Export from index.ts**: Add to barrel export
5. **Document in README**: Add to appropriate section
6. **Test thoroughly**: Unit + integration tests
7. **Consider reusability**: Can it be used elsewhere?

## Questions?

- Check the specific hook's JSDoc comments
- Review examples in the README
- Look at similar hooks for patterns
- Run tests: `npm run test`
- Check logs: Set `VITE_LOG_LEVEL=debug`
