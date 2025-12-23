# Core Updates Required for System-Tools

## Problem

The `system-tools` MCP server requires AI and agent functionality that is not currently exposed through the `systemprompt` facade.

## Required Changes

### 1. Add AI Module to Facade

**File:** `core/systemprompt/src/lib.rs`

Add AI feature and module:

```rust
#[cfg(feature = "full")]
#[cfg_attr(docsrs, doc(cfg(feature = "full")))]
pub mod ai {
    //! AI services and providers.
    pub use systemprompt_core_ai::*;
}
```

### 2. Update Facade Cargo.toml

**File:** `core/systemprompt/Cargo.toml`

Add to `[features]` section:

```toml
full = [
    # ... existing deps ...
    "dep:systemprompt-core-ai",  # ADD THIS
]
```

Add to `[dependencies]` section:

```toml
systemprompt-core-ai = { path = "../crates/domain/ai", optional = true }
```

### 3. Verify Exports

Ensure these types are accessible via facade:

- `systemprompt::ai::AiService`
- `systemprompt::ai::AiConfig`
- `systemprompt::agent::SkillService`

## Status

- [ ] AI module added to facade
- [ ] Agent module added to facade (already present)
- [ ] Facade published with updates
