# SystemPrompt API Module

This module provides the HTTP API server and gateway for SystemPrompt OS, with a clean separation of concerns and modular architecture.

## Architecture Overview

The API module is organized into distinct layers with clear responsibilities:

```
src/
├── api/                     # API Endpoint Definitions
│   ├── mod.rs              # API module exports
│   └── routes/             # Route handlers registered via module system
│       ├── mod.rs          # Routes module exports
│       ├── discovery.rs    # API discovery & well-known endpoints
│       └── services/       # Service-specific routes
│           ├── mod.rs      # Service routes exports
│           ├── agents.rs   # Agent service proxy routes
│           └── mcp.rs      # MCP service proxy routes
│
├── services/               # Business Logic Layer
│   ├── mod.rs             # Services exports
│   ├── api_lifecycle/     # API server lifecycle management
│   │   ├── mod.rs         # Lifecycle exports
│   │   ├── health.rs      # Health check endpoints
│   │   └── manager.rs     # Process management
│   ├── middleware/        # HTTP middleware components
│   │   ├── mod.rs         # Middleware exports
│   │   ├── auth.rs        # Authentication middleware
│   │   ├── cors.rs        # CORS configuration
│   │   ├── path.rs        # Path normalization
│   │   └── redirect.rs    # OAuth redirect handling
│   ├── openapi/           # OpenAPI documentation
│   │   ├── mod.rs         # OpenAPI exports
│   │   └── enhancer.rs    # OpenAPI spec enhancement
│   ├── proxy/             # Generic proxy engine (protocol-agnostic)
│   │   ├── mod.rs         # Proxy exports
│   │   ├── engine.rs      # Core proxy orchestration
│   │   ├── auth/          # Proxy authentication
│   │   │   ├── mod.rs     # Auth exports
│   │   │   ├── challenge.rs # OAuth challenge responses
│   │   │   └── validator.rs # Token validation
│   │   ├── backend/       # Backend communication
│   │   │   ├── mod.rs     # Backend exports
│   │   │   ├── request_builder.rs  # Request preparation
│   │   │   ├── response_handler.rs # Response streaming
│   │   │   └── url_resolver.rs     # URL construction
│   │   ├── client/        # HTTP client management
│   │   │   ├── mod.rs     # Client exports
│   │   │   ├── persistent.rs # Persistent connections
│   │   │   └── pool.rs    # Connection pooling
│   │   ├── discovery/     # Service discovery
│   │   │   ├── mod.rs     # Discovery exports
│   │   │   └── listing.rs # Service enumeration
│   │   └── protocol/      # Protocol-specific behavior
│   │       ├── mod.rs     # Protocol trait definitions
│   │       ├── a2a.rs     # A2A protocol implementation
│   │       └── mcp.rs     # MCP protocol implementation
│   ├── registry/          # Route registration system
│   │   ├── mod.rs         # Registry exports
│   │   ├── api_registry.rs    # Main registry implementation
│   │   ├── route_builder.rs   # Route construction logic
│   │   ├── route_inspector.rs # Route debugging utilities
│   │   └── openapi_merger.rs  # OpenAPI spec merging
│   ├── server/            # Server configuration
│   │   ├── mod.rs         # Server exports
│   │   ├── builder.rs     # Server construction
│   │   └── runner.rs      # Server execution
│   └── static_content/    # Static file serving
│       ├── mod.rs         # Static content exports
│       ├── fallback.rs    # Smart fallback handler
│       └── vite.rs        # Vite/SPA serving
│
├── models/                # Data structures & validation
│   ├── mod.rs            # Models exports
│   ├── api_lifecycle/    # Lifecycle data models
│   │   ├── mod.rs        # Lifecycle models exports
│   │   └── daemon.rs     # Daemon state models
│   ├── discovery.rs      # Discovery response models
│   ├── proxy/            # Proxy data models
│   │   ├── mod.rs        # Proxy models exports
│   │   └── client.rs     # Client configuration
│   ├── registry/         # Registry models
│   │   ├── mod.rs        # Registry models exports
│   │   └── module.rs     # Module definitions
│   └── server/           # Server models
│       ├── mod.rs        # Server models exports
│       └── config.rs     # Server configuration
│
├── cli/                  # CLI command implementations
│   ├── mod.rs           # CLI exports
│   └── commands/        # Individual commands
│       ├── mod.rs       # Commands exports
│       ├── start.rs     # Start server command
│       ├── stop.rs      # Stop server command
│       ├── restart.rs   # Restart server command
│       ├── status.rs    # Status check command
│       └── log.rs       # Log viewing command
│
├── bin/                 # Binary entry points
│   └── systemprompt-api.rs # Main API server binary
│
└── lib.rs              # Module root exports
```

## Routing Architecture

### Route Hierarchy and Precedence

The routing system follows a strict hierarchy to prevent conflicts and ensure predictable behavior:

```
Priority Order (highest to lowest):
1. API Routes (/api/*)
   - Discovery endpoint (/api)
   - OpenAPI spec (/api/v1/openapi)
   - Health check (/api/v1/health)
   - Module routes by category
2. Well-Known Routes (/.well-known/*)
   - OAuth metadata
   - Agent cards
3. Static Content (/)
   - Vite app at root
   - Asset files
4. Fallback Handler (*)
   - SPA client-side routing
   - 404 for API paths
```

### Route Categories and Mounting Points

```
/                                  # Frontend (Vite/SPA)
├── .well-known/                  # Well-known endpoints
│   ├── oauth-authorization-server  # OAuth metadata
│   ├── agent-card.json            # Default agent card
│   └── agent-card/{id}            # Specific agent cards
│
├── api                           # API discovery root
└── api/v1/                       # Versioned API
    ├── openapi                   # OpenAPI specification
    ├── health                    # Health check endpoint
    ├── core/                     # Core services
    │   ├── users/                # User management
    │   ├── config/               # Configuration
    │   └── {module}/             # Other core modules
    ├── agents/                   # Agent services (merged)
    │   ├── {agent_name}/         # Individual agent proxies
    │   └── card/{id}             # Agent card endpoints
    └── service/                  # Service proxies (merged)
        ├── mcp/                  # MCP services
        │   └── {service_name}/   # Individual MCP proxies
        └── {protocol}/           # Future protocol support
```

### Module Registration Pattern

All API modules follow this pattern:

```rust
// In api/routes/services/{module}.rs
use systemprompt_core_system::{AppContext, ServiceCategory};
use axum::{Router, routing::get};
use utoipa::OpenApi;

#[derive(Debug, Copy, Clone, OpenApi)]
#[openapi(
    paths(handler_function),
    components(schemas(ResponseModel)),
    tags((name = "module", description = "Module description"))
)]
pub struct ModuleApiDoc;

pub fn router(ctx: &AppContext) -> Router {
    Router::new()
        .route("/endpoint", get(handler_function))
        .with_state(ctx.clone())
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ModuleApiDoc::openapi()
}

// Register with inventory system
systemprompt_core_system::register_module_api!(
    "module_name",
    ServiceCategory::Core,  // or Agent, Mcp, Meta
    router,
    openapi,
    true  // auth_required
);
```

### Service Categories

- **Core** (`ServiceCategory::Core`): System management APIs
- **Agent** (`ServiceCategory::Agent`): Agent service proxies
- **MCP** (`ServiceCategory::Mcp`): MCP protocol services
- **Meta** (`ServiceCategory::Meta`): System-level routes (discovery, health)

## Proxy System Architecture

The proxy system provides a unified, protocol-agnostic engine for forwarding HTTP requests to backend services.

### Design Principles

1. **Single Engine**: One proxy implementation used by all protocols
2. **Protocol Abstraction**: Protocol-specific behavior via traits
3. **Modular Components**: Auth, client management, request/response handling separated
4. **No Duplication**: Shared logic, protocol-specific customization

### Usage Pattern

API routes use the proxy engine like this:

```rust
// In api/routes/services/agents.rs
use crate::services::proxy::ProxyEngine;

pub async fn handle_agent_proxy(
    Path(service_name): Path<String>,
    State(ctx): State<AppContext>,
    request: Request<Body>,
) -> impl IntoResponse {
    let engine = ProxyEngine::new();
    engine.proxy_request(&service_name, "", request, ctx).await
}
```

## Service Registration

Services are registered via the inventory pattern:

```rust
inventory::submit! {
    ModuleApiRegistration {
        module_name: "example",
        category: ServiceCategory::Core,
        router_fn: router,
        openapi_fn: openapi,
        auth_required: true,
    }
}
```

This enables:
- Automatic route mounting
- OpenAPI documentation generation
- Authentication middleware application
- Service discovery

## Authentication

Category-based authentication is applied automatically:

- **Core routes** (`/api/v1/core/*`): Require authentication
- **Agent routes** (`/api/v1/agents/*`): Service-specific auth
- **MCP routes** (`/api/v1/mcp/*`): Service-specific auth
- **Meta routes**: Generally public

## Common Routing Problems and Solutions

### Problem: Routes returning 404 unexpectedly

**Causes:**
- Routes mounted in wrong order (fallback taking precedence)
- Trailing slash inconsistency
- Middleware normalizing paths incorrectly

**Solution:**
- Mount routes in strict priority order: API → Well-known → Static → Fallback
- Use `PathMiddleware` to normalize trailing slashes
- Test with and without trailing slashes

### Problem: Fallback handler serving API routes

**Causes:**
- Fallback handler not checking registered API paths
- Routes not properly registered with inventory system

**Solution:**
- Implement smart fallback that checks for API path prefixes
- Return 404 for unmatched API routes instead of serving SPA

### Problem: Category routes conflicting

**Causes:**
- Dual mounting of routes (e.g., at `/api/v1/service` and `/api/v1/agents`)
- Meta routes with empty names overriding other routes

**Solution:**
- Single mounting point per category
- Clear category boundaries
- Meta routes mount at specific paths

## Key Benefits

1. **Clean Separation**: Routes, business logic, and data models are distinct
2. **No Duplication**: Single proxy engine serves all protocols
3. **Modular**: Components can be tested and modified independently
4. **Extensible**: New protocols can be added via traits
5. **Consistent**: All modules follow the same registration pattern
6. **Predictable**: Clear route hierarchy prevents conflicts

## Anti-Patterns Avoided

- ❌ Hardcoded routes in service layers
- ❌ Duplicate proxy implementations per protocol
- ❌ Mixed routing and business logic
- ❌ Protocol-specific snowflake implementations
- ❌ Ambiguous route precedence
- ❌ Fallback handlers that mask API errors

The architecture ensures maintainability, testability, and consistency across the entire API surface.