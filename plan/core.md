# Core CLI: Perfect Release Process

## Current State

The CLI has several issues preventing a clean release process:

1. **Hardcoded paths** - Looks for `infrastructure/` directory (removed)
2. **Profile paths ignored** - Loads profiles but reads raw env vars instead
3. **Split responsibilities** - Deploy logic scattered between justfile and CLI

## Target State

```
just deploy  →  systemprompt cloud deploy  →  Done
```

The CLI owns the entire process. The justfile is a thin passthrough.

---

## Bug 1: Hardcoded Infrastructure Path

**File**: `core/crates/entry/cli/src/cloud/deploy.rs`

**Error**:
```
Error: Project root not found. Expected to find 'infrastructure' directory at: /var/www/html/systemprompt-template
```

**Current** (wrong):
```rust
fn find_project_root() -> Result<PathBuf> {
    let current = std::env::current_dir()?;
    for ancestor in current.ancestors() {
        if ancestor.join("infrastructure").is_dir() {
            return Ok(ancestor.to_path_buf());
        }
    }
    bail!("Project root not found. Expected to find 'infrastructure' directory");
}
```

**Fix**:
```rust
fn find_project_root() -> Result<PathBuf> {
    let current = std::env::current_dir()?;
    for ancestor in current.ancestors() {
        // Look for .systemprompt/ directory (template convention)
        if ancestor.join(".systemprompt").is_dir() {
            return Ok(ancestor.to_path_buf());
        }
        // Fallback: look for core/ submodule
        if ancestor.join("core").is_dir() && ancestor.join("services").is_dir() {
            return Ok(ancestor.to_path_buf());
        }
    }
    bail!("Project root not found. Expected .systemprompt/ directory or core/ + services/");
}
```

---

## Bug 2: Hardcoded Dockerfile Path

**File**: `core/crates/entry/cli/src/cloud/deploy.rs`

**Current** (wrong):
```rust
let dockerfile = project_root.join("infrastructure/docker/app.Dockerfile");
```

**Fix**:
```rust
let dockerfile = project_root.join(".systemprompt/Dockerfile");
```

---

## Bug 3: Profile Paths Not Used

**File**: `core/crates/entry/cli/src/build/web.rs:36`

**Current** (wrong):
```rust
pub async fn execute(args: &WebArgs) -> Result<()> {
    let web_path = std::env::var("SYSTEMPROMPT_WEB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("core/web"));
    // ...
}
```

**Fix**:
```rust
use systemprompt_models::profile_bootstrap::ProfileBootstrap;

pub async fn execute(args: &WebArgs) -> Result<()> {
    let profile = ProfileBootstrap::get()
        .ok_or_else(|| anyhow!("No profile loaded. Run from project root or set SYSTEMPROMPT_PROFILE"))?;

    let web_path = profile.paths.web
        .as_ref()
        .ok_or_else(|| anyhow!("Profile missing 'paths.web' configuration"))?;

    if !web_path.exists() {
        bail!("Web path does not exist: {}", web_path.display());
    }
    // ...
}
```

---

## Bug 4: Deploy Should Own Docker Build

**File**: `core/crates/entry/cli/src/cloud/deploy.rs`

The CLI should handle the complete deploy flow:

```rust
pub async fn execute(args: &DeployArgs) -> Result<()> {
    let project_root = find_project_root()?;
    let credentials = load_credentials()?;

    // 1. Validate pre-built artifacts
    let binary = project_root.join("target/release/systemprompt");
    let web_dist = project_root.join("core/web/dist");

    if !binary.exists() {
        bail!(
            "Release binary not found at: {}\n\
             Run: cargo build --release --manifest-path=core/Cargo.toml --bin systemprompt",
            binary.display()
        );
    }

    if !web_dist.exists() {
        bail!(
            "Web assets not found at: {}\n\
             Run: npm run build --prefix core/web",
            web_dist.display()
        );
    }

    // 2. Generate image tag
    let tag = args.tag.clone().unwrap_or_else(|| {
        let timestamp = chrono::Utc::now().timestamp();
        let git_sha = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        format!("deploy-{}-{}", timestamp, git_sha)
    });

    let image = format!("registry.fly.io/{}:{}", credentials.app_id, tag);

    // 3. Build Docker image
    info!("Building Docker image: {}", image);
    let dockerfile = project_root.join(".systemprompt/Dockerfile");

    if !dockerfile.exists() {
        bail!("Dockerfile not found at: {}", dockerfile.display());
    }

    let status = Command::new("docker")
        .args(["build", "-f"])
        .arg(&dockerfile)
        .args(["-t", &image, "."])
        .current_dir(&project_root)
        .status()?;

    if !status.success() {
        bail!("Docker build failed");
    }

    // 4. Push to registry (unless --skip-push)
    if !args.skip_push {
        info!("Pushing image to registry...");
        let status = Command::new("docker")
            .args(["push", &image])
            .status()?;

        if !status.success() {
            bail!("Docker push failed. Ensure you're logged in: docker login registry.fly.io");
        }
    }

    // 5. Trigger deployment via API
    info!("Triggering deployment...");
    let client = SystemPromptClient::new(&credentials)?;
    client.deploy(&tag).await?;

    info!("Deployment initiated: {}", tag);
    Ok(())
}
```

---

## Bug 5: Artifact Paths Should Come From Profile

**File**: `core/crates/entry/cli/src/cloud/deploy.rs`

Instead of hardcoding paths, read from profile:

```rust
pub async fn execute(args: &DeployArgs) -> Result<()> {
    let profile = ProfileBootstrap::get()
        .ok_or_else(|| anyhow!("No profile loaded"))?;

    let binary = profile.paths.release_binary
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release/systemprompt"));

    let web_dist = profile.paths.web_dist
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("core/web/dist"));

    let dockerfile = profile.paths.dockerfile
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".systemprompt/Dockerfile"));

    // ... rest of deploy logic
}
```

**Profile schema addition** (`services/profiles/*.profile.yml`):
```yaml
paths:
  services: ./services
  web: ./core/web
  web_dist: ./core/web/dist
  release_binary: ./target/release/systemprompt
  dockerfile: ./.systemprompt/Dockerfile
```

---

## Summary of Files to Modify

| File | Change |
|------|--------|
| `crates/entry/cli/src/cloud/deploy.rs` | Fix project root detection, own Docker build/push, use profile paths |
| `crates/entry/cli/src/build/web.rs` | Use profile paths instead of env vars |
| `crates/entry/cli/src/build/fly.rs` | Use profile paths instead of hardcoded paths |
| `crates/shared/models/src/profile.rs` | Add `release_binary`, `web_dist`, `dockerfile` to paths schema |

---

## Result

After these fixes:

**justfile** (entire file):
```just
set dotenv-load

CLI := "target/debug/systemprompt"

start:
    {{CLI}} services start

deploy:
    {{CLI}} cloud deploy

systemprompt *ARGS:
    {{CLI}} {{ARGS}}
```

**User workflow**:
```bash
# Build release
cd core && cargo build --release --bin systemprompt
npm run build --prefix core/web
cp target/release/systemprompt ../target/release/

# Deploy (CLI handles everything)
just deploy
```

All intelligence in the CLI. Justfile is a thin wrapper. Dockerfile is dumb packaging.
