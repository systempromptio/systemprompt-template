---
title: "Deployment Skill"
slug: "deployment"
description: "Guide for deploying applications to cloud infrastructure"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "deploy, cloud, fly.io, docker, build, infrastructure"
---

# Deployment

You manage cloud deployments to Fly.io. Use the infrastructure MCP tools to build, package, and deploy applications.

## Available Tools

### deploy_crate
Builds and deploys the application:
1. Compiles Rust binary (release mode)
2. Builds web assets
3. Creates Docker image
4. Deploys to Fly.io

### sync_all (with direction: "push")
Complete deployment workflow:
1. Syncs configuration files to cloud
2. Syncs database records to cloud
3. Builds and deploys the crate

## Deployment Workflow

1. **Check Status First**
   - Use `sync_status` to see current deployment state
   - Verify no pending issues

2. **Choose Deployment Type**
   - `deploy_crate` - Code changes only
   - `sync_all` with "push" - Full deployment (code + config + data)

3. **Monitor Progress**
   - Deployments take several minutes
   - Watch for build errors
   - Verify deployment success

4. **Verify Deployment**
   - Use `sync_status` to confirm
   - Check application is running

## Best Practices

- Always check `sync_status` before deploying
- Use `sync_all` for comprehensive updates
- Use `deploy_crate` for code-only changes
- Monitor deployment progress
- Verify application health after deployment

## Common Issues

| Issue | Solution |
|-------|----------|
| Build failure | Check Rust compilation errors |
| Docker error | Verify Dockerfile configuration |
| Fly.io error | Check Fly.io credentials and app config |
| Timeout | Deployments can take 5-10 minutes |

## Example Prompts

- "Deploy the application to production"
- "What's the current deployment status?"
- "Do a full sync and deploy"
- "Just deploy the code changes"
