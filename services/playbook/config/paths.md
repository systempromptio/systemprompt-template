---
title: "Paths Configuration"
description: "Configure directory locations for system files, services, and storage."
author: "SystemPrompt"
slug: "config-paths"
keywords: "paths, directories, system, services, bin, storage, resolution"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Paths Configuration

Configure directory locations for system files, services, and storage.

> **Help**: `{ "command": "admin config paths show" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

PathsConfig defines directory locations for system files, services, binaries, and optional storage.

---

## PathsConfig Struct

**Source**: `crates/shared/models/src/profile/paths.rs:4-18`

```rust
pub struct PathsConfig {
    pub system: String,                        // Required: System root directory
    pub services: String,                      // Required: Services directory
    pub bin: String,                           // Required: Binaries directory
    pub web_path: Option<String>,              // Optional: Web output path
    pub storage: Option<String>,               // Optional: File storage
    pub geoip_database: Option<String>,        // Optional: MaxMind .mmdb path
}
```

### Field Details

| Field | Required | Description |
|-------|----------|-------------|
| `system` | Yes | Project root directory |
| `services` | Yes | Services configuration directory |
| `bin` | Yes | Compiled binaries directory |
| `web_path` | No | Web assets output location |
| `storage` | No | File storage directory |
| `geoip_database` | No | MaxMind GeoIP database file |

---

## Derived Paths

PathsConfig provides methods that derive additional paths from the base configuration.

**Source**: `crates/shared/models/src/profile/paths.rs:20-67`

### Method Reference

| Method | Returns | Purpose |
|--------|---------|---------|
| `skills()` | `{services}/skills` | Skill definitions |
| `config()` | `{services}/config/config.yaml` | Main config file |
| `ai_config()` | `{services}/ai/config.yaml` | AI provider config |
| `content_config()` | `{services}/content/config.yaml` | Content routing config |
| `web_config()` | `{services}/web/config.yaml` | Web generation config |
| `web_metadata()` | `{services}/web/metadata.yaml` | Web metadata |
| `web_path_resolved()` | `web_path` or `{system}/web` | Resolved web path |
| `storage_resolved()` | `storage` if set | Resolved storage path |
| `geoip_database_resolved()` | `geoip_database` if set | Resolved GeoIP path |

### Implementation

```rust
impl PathsConfig {
    pub fn skills(&self) -> String {
        format!("{}/skills", self.services)
    }

    pub fn config(&self) -> String {
        format!("{}/config/config.yaml", self.services)
    }

    pub fn ai_config(&self) -> String {
        format!("{}/ai/config.yaml", self.services)
    }

    pub fn content_config(&self) -> String {
        format!("{}/content/config.yaml", self.services)
    }

    pub fn web_config(&self) -> String {
        format!("{}/web/config.yaml", self.services)
    }

    pub fn web_metadata(&self) -> String {
        format!("{}/web/metadata.yaml", self.services)
    }

    pub fn web_path_resolved(&self) -> String {
        self.web_path.clone().unwrap_or_else(|| {
            format!("{}/web", self.system)
        })
    }

    pub fn storage_resolved(&self) -> Option<&str> {
        self.storage.as_deref()
    }

    pub fn geoip_database_resolved(&self) -> Option<&str> {
        self.geoip_database.as_deref()
    }
}
```

---

## Path Resolution Functions

**Source**: `crates/shared/models/src/profile/paths.rs:69-108`

### resolve_path

Resolves relative paths against a base directory:

```rust
pub fn resolve_path(base: &Path, path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_string_lossy().to_string()
    } else {
        base.join(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| base.join(path).to_string_lossy().to_string())
    }
}
```

### expand_home

Expands `~/` to home directory:

```rust
pub fn expand_home(path_str: &str) -> PathBuf {
    if path_str.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE")) {
            return PathBuf::from(home).join(&path_str[2..]);
        }
    }
    PathBuf::from(path_str)
}
```

### resolve_with_home

Combines home expansion with relative resolution:

```rust
pub fn resolve_with_home(base: &Path, path_str: &str) -> PathBuf {
    let expanded = expand_home(path_str);
    if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    }
}
```

---

## Path Resolution in Profile Loading

When a profile loads, paths are resolved relative to the profile directory:

```rust
impl PathsConfig {
    pub fn resolve_relative_to(&mut self, base: &Path) {
        self.system = resolve_path(base, &self.system);
        self.services = resolve_path(base, &self.services);
        self.bin = resolve_path(base, &self.bin);

        if let Some(ref path) = self.web_path {
            self.web_path = Some(resolve_path(base, path));
        }
        if let Some(ref path) = self.storage {
            self.storage = Some(resolve_path(base, path));
        }
        if let Some(ref path) = self.geoip_database {
            self.geoip_database = Some(resolve_path(base, path));
        }
    }
}
```

---

## Cloud vs Local Validation

**Source**: `crates/shared/models/src/profile/validation.rs:34-112`

### Local Profile Validation

For `target: local`:

```rust
fn validate_local_paths(&self) -> Result<()> {
    // Required paths must exist
    let paths_to_check = [
        ("system", &self.paths.system),
        ("services", &self.paths.services),
        ("bin", &self.paths.bin),
    ];

    for (name, path) in paths_to_check {
        if !Path::new(path).exists() {
            return Err(ProfileError::PathNotFound(name, path.clone()));
        }
    }

    // Optional paths must exist if specified
    if let Some(ref path) = self.paths.storage {
        if !Path::new(path).exists() {
            return Err(ProfileError::PathNotFound("storage", path.clone()));
        }
    }
    if let Some(ref path) = self.paths.geoip_database {
        if !Path::new(path).exists() {
            return Err(ProfileError::PathNotFound("geoip_database", path.clone()));
        }
    }
    if let Some(ref path) = self.paths.web_path {
        if !Path::new(path).exists() {
            return Err(ProfileError::PathNotFound("web_path", path.clone()));
        }
    }

    Ok(())
}
```

### Cloud Profile Validation

For `target: cloud`:

```rust
fn validate_cloud_paths(&self) -> Result<()> {
    // Required paths must be non-empty and start with /app
    let required = [
        ("system", &self.paths.system),
        ("services", &self.paths.services),
        ("bin", &self.paths.bin),
    ];

    for (name, path) in required {
        if path.is_empty() {
            return Err(ProfileError::EmptyPath(name));
        }
        if !path.starts_with("/app") {
            return Err(ProfileError::CloudPathMustStartWithApp(name, path.clone()));
        }
    }

    // web_path must start with /app/web (not /app/services/web)
    if let Some(ref path) = self.paths.web_path {
        if !path.starts_with("/app/web") {
            return Err(ProfileError::CloudWebPathInvalid(path.clone()));
        }
        if path.starts_with("/app/services/web") {
            return Err(ProfileError::CloudWebPathInServicesDir);
        }
    }

    Ok(())
}
```

---

## Configuration Examples

### Local Development

```yaml
paths:
  system: "/home/user/my-project"
  services: "/home/user/my-project/services"
  bin: "/home/user/my-project/target/release"
  web_path: "/home/user/my-project/web"
  storage: "/home/user/my-project/storage"
  geoip_database: "/home/user/my-project/data/GeoLite2-City.mmdb"
```

### Relative Paths (Recommended)

```yaml
# Profile at .systemprompt/profiles/local/profile.yaml
paths:
  system: "../../.."               # Resolves to project root
  services: "../../../services"    # Resolves to services/
  bin: "../../../target/release"   # Resolves to target/release/
  web_path: "../../../web"
  storage: "../../../storage"
```

### Home Directory

```yaml
paths:
  system: "~/projects/my-project"
  services: "~/projects/my-project/services"
  bin: "~/projects/my-project/target/release"
```

### Cloud Deployment

```yaml
paths:
  system: "/app"
  services: "/app/services"
  bin: "/app/bin"
  web_path: "/app/web"
```

---

## Directory Structure

Expected directory layout:

```
{system}/
├── services/                      # {services}
│   ├── config/
│   │   └── config.yaml           # paths.config()
│   ├── ai/
│   │   └── config.yaml           # paths.ai_config()
│   ├── content/
│   │   └── config.yaml           # paths.content_config()
│   ├── web/
│   │   ├── config.yaml           # paths.web_config()
│   │   └── metadata.yaml         # paths.web_metadata()
│   └── skills/                   # paths.skills()
│       └── *.yaml
├── target/release/               # {bin}
│   └── systemprompt
├── web/                          # {web_path}
│   └── dist/
├── storage/                      # {storage}
│   └── files/
└── data/
    └── GeoLite2-City.mmdb       # {geoip_database}
```

---

## Environment Variables

When using `Profile::from_env()`:

| Env Variable | Maps To |
|--------------|---------|
| `SYSTEM_PATH` | `paths.system` |
| `SYSTEMPROMPT_SERVICES_PATH` | `paths.services` |
| `BIN_PATH` | `paths.bin` |
| `SYSTEMPROMPT_WEB_PATH` | `paths.web_path` |
| `STORAGE_PATH` | `paths.storage` |
| `GEOIP_DATABASE_PATH` | `paths.geoip_database` |

---

## Config Access

After bootstrap, paths are available via Config struct:

```rust
let config = Config::get()?;
println!("System: {}", config.system_path);
println!("Services: {}", config.services_path);
println!("Bin: {}", config.bin_path);
println!("Skills: {}", config.skills_path);
println!("Settings: {}", config.settings_path);
println!("Content Config: {}", config.content_config_path);
println!("Web: {}", config.web_path);
println!("Web Config: {}", config.web_config_path);
println!("Web Metadata: {}", config.web_metadata_path);
if let Some(geoip) = &config.geoip_database_path {
    println!("GeoIP: {}", geoip);
}
```

---

## Troubleshooting

**"Path does not exist"**
- For local profiles, all required paths must exist
- Create missing directories before starting
- Check for typos in path strings

**"Path must start with /app"**
- Cloud profiles require `/app` prefix
- Ensure `target: cloud` has cloud-style paths

**"web_path invalid"**
- Cloud web_path must start with `/app/web`
- Cannot be `/app/services/web`

**"Cannot canonicalize path"**
- Path contains symlinks that don't resolve
- Directory doesn't exist yet
- Check permissions

**"Home directory not found"**
- `HOME` or `USERPROFILE` env var not set
- Use absolute paths instead

---

## Quick Reference

### Required Paths

| Path | Local | Cloud |
|------|-------|-------|
| `system` | Must exist | Must start with `/app` |
| `services` | Must exist | Must start with `/app` |
| `bin` | Must exist | Must start with `/app` |

### Optional Paths

| Path | Default | Description |
|------|---------|-------------|
| `web_path` | `{system}/web` | Web output |
| `storage` | None | File storage |
| `geoip_database` | None | MaxMind database |

### Derived Paths

| Method | Pattern |
|--------|---------|
| `skills()` | `{services}/skills` |
| `config()` | `{services}/config/config.yaml` |
| `ai_config()` | `{services}/ai/config.yaml` |
| `content_config()` | `{services}/content/config.yaml` |
| `web_config()` | `{services}/web/config.yaml` |
| `web_metadata()` | `{services}/web/metadata.yaml` |