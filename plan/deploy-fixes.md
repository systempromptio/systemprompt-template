# Required Core Fixes for Deploy Pipeline

## Current Deploy Failures

When deploying to Fly.io, the app fails with these errors:

```
Profile 'prod' validation failed:
    - Invalid database URL: must start with 'postgres://' or 'postgresql://'
    Got: ${DATABASE_URL}
    - Core path does not exist: /app/core
    - paths.web_path does not exist: /app/core/web
    - Security jwt_secret must be at least 32 characters
```

## Root Causes & Exact Code Changes Required

### 1. Environment Variable Substitution Not Working
**Location**: `core/crates/shared/models/src/profile.rs:605-619`

Current code reads YAML and parses directly without substituting `${VAR}`:
```rust
pub fn load_from_path(profile_path: &Path) -> Result<Self> {
    let content = std::fs::read_to_string(profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;

    let mut profile: Self = serde_yaml::from_str(&content)  // <-- No env substitution!
        .with_context(|| format!("Failed to parse profile: {}", profile_path.display()))?;
    ...
}
```

**Fix Required**: Add substitution before parsing:
```rust
pub fn load_from_path(profile_path: &Path) -> Result<Self> {
    let content = std::fs::read_to_string(profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;

    // Substitute environment variables
    let content = substitute_env_vars(&content);

    let mut profile: Self = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse profile: {}", profile_path.display()))?;
    ...
}

fn substitute_env_vars(content: &str) -> String {
    let re = regex::Regex::new(r"\$\{(\w+)\}").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let var_name = &caps[1];
        std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
    }).to_string()
}
```

### 2. Profile Name vs Full Path Resolution
**Location**: `core/crates/shared/models/src/profile_bootstrap.rs:50-57`

Current code takes SYSTEMPROMPT_PROFILE directly as a path:
```rust
let path = match profile_path {
    Some(p) => p.to_path_buf(),
    None => {
        let path_str = std::env::var("SYSTEMPROMPT_PROFILE")
            .map_err(|_| ProfileBootstrapError::PathNotSet)?;
        std::path::PathBuf::from(path_str)  // <-- Assumes full path!
    },
};
```

**Fix Required**: Resolve profile names to full paths:
```rust
let path = match profile_path {
    Some(p) => p.to_path_buf(),
    None => {
        let path_str = std::env::var("SYSTEMPROMPT_PROFILE")
            .map_err(|_| ProfileBootstrapError::PathNotSet)?;

        // If it's a full path, use it directly
        if path_str.contains('/') || path_str.ends_with(".yml") {
            std::path::PathBuf::from(path_str)
        } else {
            // Resolve profile name to full path
            let services_path = std::env::var("SYSTEMPROMPT_SERVICES_PATH")
                .unwrap_or_else(|_| "/app/services".to_string());
            std::path::PathBuf::from(&services_path)
                .join("profiles")
                .join(format!("{}.profile.yml", path_str))
        }
    },
};
```

### 3. Path Validation Too Strict
**Location**: `core/crates/shared/models/src/profile.rs:697-704` and similar

Current code validates paths exist:
```rust
if self.paths.core.is_empty() {
    errors.push("Paths core is required".to_string());
} else if !Path::new(&self.paths.core).exists() {
    errors.push(format!("Core path does not exist: {}", self.paths.core));
}
```

**Fix Required**: Skip existence checks for optional paths in production:
```rust
// Core path is optional in production containers
if !self.paths.core.is_empty() && !Path::new(&self.paths.core).exists() {
    // Only warn, don't error - core is for local dev only
    tracing::warn!("Core path does not exist (expected in production): {}", self.paths.core);
}
```

Or make these paths truly optional:
```rust
pub struct PathsConfig {
    pub core: Option<String>,  // Changed from String to Option<String>
    // ...
}
```

## Summary of Core Changes

| File | Line | Change |
|------|------|--------|
| `profile.rs` | 605-619 | Add `substitute_env_vars()` call before YAML parsing |
| `profile_bootstrap.rs` | 50-57 | Add profile name → path resolution |
| `profile.rs` | 697-704+ | Make `core`, `web_path` existence checks non-fatal |

## Template Changes (This Repo) - DONE

1. ✅ Updated `services/profiles/prod.profile.yml`:
   - Changed `web_path: /app/web`
   - Changed `core: /app` (points to existing directory)
   - Added `web_dist: /app/web`

2. ✅ Fixed `.systemprompt/Dockerfile` - removed `api` from CMD
