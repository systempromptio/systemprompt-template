---
title: "Files"
description: "Upload, serve, permission. File storage that works without S3 configuration or CDN setup."
author: "SystemPrompt Team"
slug: "services/files"
keywords: "files, storage, upload, serve, permissions"
image: "/files/images/docs/services-files.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Files

SystemPrompt includes file storage that works out of the box. Upload files, serve them to users, manage permissions—all without configuring S3 buckets, CDN distributions, or external services. For production deployments, you can optionally integrate with cloud storage providers.

The file system is designed for AI workloads. Agents can upload artifacts, store generated content, and share files between sessions. Every file operation respects tenant boundaries and permission scopes.

## File Storage Architecture

The file storage system has three layers that work together to provide a unified interface regardless of the underlying storage backend.

**Service layer**: The `systemprompt-files` crate provides the public API. Methods like `upload()`, `download()`, `list()`, and `delete()` work identically whether files are stored locally or in the cloud.

**Provider layer**: Storage providers implement the actual storage operations. The local provider stores files on the filesystem. Cloud providers integrate with S3-compatible services. The provider is selected based on configuration.

**Permission layer**: Every file operation checks permissions. Files have owners (users or system), visibility (public or private), and access scopes. The permission layer integrates with the authentication system.

```
API Request
    ↓
File Service (systemprompt-files)
    ↓
Permission Check
    ↓
Storage Provider (local/S3/etc)
    ↓
Filesystem or Cloud Storage
```

## Upload and Manage Files

Files are uploaded through the API or CLI. Each upload creates a file record in the database and stores the content in the configured backend.

**CLI upload:**

```bash
# Upload a file
systemprompt core files upload /path/to/local/file.pdf

# Upload with metadata
systemprompt core files upload /path/to/image.png \
  --content-type "image/png" \
  --visibility public

# Upload to a specific path
systemprompt core files upload /path/to/doc.pdf \
  --destination "/documents/reports/2026/"
```

**File metadata:**

| Property | Description |
|----------|-------------|
| `id` | Unique file identifier |
| `tenant_id` | Owning tenant |
| `owner_id` | User who uploaded |
| `path` | Logical file path |
| `filename` | Original filename |
| `content_type` | MIME type |
| `size` | File size in bytes |
| `visibility` | `public` or `private` |
| `checksum` | Content hash for integrity |
| `created_at` | Upload timestamp |

**Listing files:**

```bash
# List all files
systemprompt core files list

# List files in a directory
systemprompt core files list --path "/documents/"

# List with details
systemprompt core files list --format detailed
```

**Deleting files:**

```bash
# Delete a specific file
systemprompt core files delete <file_id>

# Delete by path
systemprompt core files delete --path "/documents/old-report.pdf"
```

## Serve Files with Permissions

Files are served through the API with automatic permission checking. Public files are accessible without authentication. Private files require valid authentication and appropriate scopes.

**File serving endpoints:**

| Endpoint | Description |
|----------|-------------|
| `/files/<path>` | Public file serving |
| `/api/v1/files/<id>` | API file access |
| `/api/v1/files/<id>/download` | Force download |

**Public files:**

Files with `visibility: public` are served directly. These are suitable for static assets, public images, and downloadable resources.

```bash
# Make a file public
systemprompt core files update <file_id> --visibility public

# File is now accessible at /files/<path>
```

**Private files:**

Files with `visibility: private` require authentication. The API checks the requestor's token and verifies they have access to the file's tenant.

```bash
# Access private file via API
curl -H "Authorization: Bearer <token>" \
  https://your-domain.com/api/v1/files/<file_id>
```

**Signed URLs:**

For temporary access to private files, generate signed URLs that expire after a specified duration.

```bash
# Generate a signed URL (expires in 1 hour)
systemprompt core files sign <file_id> --expires 3600

# Output: https://your-domain.com/files/signed/<token>
```

Signed URLs are useful for sharing files with users who don't have direct API access, or for embedding private images in emails.

## Configure Storage Backends

The default configuration uses local filesystem storage. For production deployments, configure a cloud storage provider for durability and scalability.

**Local storage (default):**

```yaml
# Profile configuration
files:
  provider: local
  local:
    root: "./storage/files"
    max_file_size_mb: 100
```

Local storage keeps files in the specified directory. Files are organized by tenant and path. This is suitable for development and single-server deployments.

**S3-compatible storage:**

```yaml
# Profile configuration
files:
  provider: s3
  s3:
    bucket: "your-bucket-name"
    region: "us-east-1"
    endpoint: null  # Use default AWS endpoint
    access_key_id: ${AWS_ACCESS_KEY_ID}
    secret_access_key: ${AWS_SECRET_ACCESS_KEY}
```

S3 storage works with AWS S3 and any S3-compatible service (MinIO, DigitalOcean Spaces, Backblaze B2). Files are stored with tenant prefixes for isolation.

**Storage organization:**

Regardless of backend, files are organized consistently:

```
<storage_root>/
├── tenants/
│   └── <tenant_id>/
│       ├── files/
│       │   └── <user_uploads>
│       └── artifacts/
│           └── <agent_generated>
├── public/
│   └── <shared_assets>
└── system/
    └── <platform_files>
```

## File Types and Content

SystemPrompt handles various file types with appropriate processing.

**Images:**

Images can be resized and optimized on upload. Thumbnail generation is available for preview purposes.

```bash
# Upload image with processing
systemprompt core files upload photo.jpg \
  --process resize:800x600 \
  --process optimize
```

**Documents:**

PDF and document files are stored as-is. Text extraction is available for indexing and search.

**Artifacts:**

AI-generated content (code, reports, visualizations) can be stored as artifacts with metadata linking them to conversations and agents.

## Storage Quotas

Tenants can have storage quotas to manage resource usage.

**Quota configuration:**

```yaml
# Tenant settings (via cloud API)
tenant:
  settings:
    storage_quota_gb: 100
    max_file_size_mb: 50
```

**Checking usage:**

```bash
# Show storage usage
systemprompt core files stats

# Output:
# Total: 2.4 GB / 100 GB (2.4%)
# Files: 1,247
# Largest: document.pdf (156 MB)
```

## File Operations for AI Agents

Agents can interact with files through MCP tools. This enables workflows where agents generate content, store artifacts, and retrieve previous work.

**Agent file operations:**

- **Upload**: Store generated content (reports, code, images)
- **Download**: Retrieve files for processing
- **List**: Browse available files
- **Search**: Find files by name or metadata

The MCP tools respect the same permission model as the API. Agents can only access files within their authorized scope.

## Configuration Reference

| Item | Location | Description |
|------|----------|-------------|
| Provider | Profile (`files.provider`) | Storage backend selection |
| Local root | Profile (`files.local.root`) | Local storage directory |
| S3 bucket | Profile (`files.s3.bucket`) | Cloud storage bucket |
| Max size | Profile (`files.max_file_size_mb`) | Upload size limit |
| Quotas | Tenant settings | Per-tenant storage limits |

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt core files list` | List files with pagination and filtering |
| `systemprompt core files show <id>` | Show detailed file information |
| `systemprompt core files upload <path>` | Upload a file from the local filesystem |
| `systemprompt core files delete <id>` | Delete a file |
| `systemprompt core files validate <path>` | Validate a file before upload |
| `systemprompt core files config` | Show file upload configuration |
| `systemprompt core files search <pattern>` | Search files by path pattern |
| `systemprompt core files stats` | Show file storage statistics |
| `systemprompt core files ai` | AI-generated images operations |

See `systemprompt core files <command> --help` for detailed options.