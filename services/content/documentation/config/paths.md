---
title: "Paths Configuration"
description: "Directory paths for system files, services, binaries, and optional storage."
author: "SystemPrompt Team"
slug: "config/paths"
keywords: "paths, directories, system, services, bin, storage"
image: "/files/images/docs/config-paths.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Paths Configuration

Paths define where SystemPrompt finds system files, services, binaries, and stores data.

## Configuration

```yaml
# .systemprompt/profiles/local/profile.yaml
paths:
  system: "/path/to/project"
  services: "/path/to/project/services"
  bin: "/path/to/project/target/release"
  web_path: "/path/to/project/web"
  storage: "/path/to/project/storage"
  geoip_database: "/path/to/GeoLite2-City.mmdb"
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `system` | Yes | Project root directory |
| `services` | Yes | Services configuration |
| `bin` | Yes | Compiled binaries |
| `web_path` | No | Web output location |
| `storage` | No | File storage |
| `geoip_database` | No | MaxMind GeoIP database |

## Relative Paths

Paths are resolved relative to the profile directory:

```yaml
# Profile at .systemprompt/profiles/local/profile.yaml
paths:
  system: "../../.."               # Project root
  services: "../../../services"
  bin: "../../../target/release"
```

## Derived Paths

SystemPrompt derives additional paths from your configuration:

| Derived Path | Pattern |
|--------------|---------|
| Skills | `{services}/skills` |
| Config | `{services}/config/config.yaml` |
| AI Config | `{services}/ai/config.yaml` |
| Content Config | `{services}/content/config.yaml` |
| Web Config | `{services}/web/config.yaml` |
| Web Metadata | `{services}/web/metadata.yaml` |

## Local vs Cloud

### Local Profiles

All required paths must exist on the filesystem.

### Cloud Profiles

All paths must start with `/app`:

```yaml
paths:
  system: "/app"
  services: "/app/services"
  bin: "/app/bin"
  web_path: "/app/web"
```

## Directory Structure

```
{system}/
├── services/                      # {services}
│   ├── config/config.yaml
│   ├── content/config.yaml
│   ├── web/config.yaml
│   └── skills/
├── target/release/               # {bin}
│   └── systemprompt
├── web/                          # {web_path}
└── storage/                      # {storage}
```

See the [Paths Configuration Playbook](/playbooks/config-paths) for detailed technical information.