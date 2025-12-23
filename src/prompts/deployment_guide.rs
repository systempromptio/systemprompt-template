#[must_use]
pub fn build_deployment_guide_prompt(environment: &str, include_rollback: bool) -> String {
    let mut prompt = format!(
        r"# SystemPrompt Deployment Guide - {environment} Environment

## Overview

This guide walks through deploying a SystemPrompt application to the {environment} environment using the infrastructure MCP server tools.

## Prerequisites

Before deploying, ensure you have:
- Valid API credentials configured (`~/.systemprompt/credentials.json` or environment variables)
- Local codebase up to date with the latest changes
- All tests passing locally
- Database migrations ready (if applicable)

## Deployment Steps

### Step 1: Check Sync Status

First, verify your sync configuration is correct:

```
Use the sync_status tool to check:
- Cloud connectivity
- Current deployment status
- Last sync information
```

### Step 2: Sync Files (Dry Run)

Preview file changes before applying:

```
Use sync_files with:
- direction: 'push'
- dry_run: true

Review the files that will be synced.
```

### Step 3: Sync Database (Dry Run)

Preview database changes:

```
Use sync_database with:
- direction: 'push'
- dry_run: true
- tables: ['agents', 'skills', 'contexts'] (or leave empty for all)

Review the records that will be synced.
```

### Step 4: Execute Full Sync

If the dry runs look good, execute the actual sync:

```
Use sync_all with:
- direction: 'push'
- dry_run: false

This will:
1. Sync all configuration files
2. Sync database records
3. Build and deploy the application
```

### Step 5: Verify Deployment

After deployment completes:
1. Check the deployment URL returned
2. Verify the application is running
3. Test critical functionality

"
    );

    if include_rollback {
        prompt.push_str(
            r"
## Rollback Procedures

If issues are detected after deployment:

### Quick Rollback

1. Use sync_files with direction: 'pull' to restore previous file state
2. Use sync_database with direction: 'pull' to restore database state

### Full Rollback

Use sync_all with:
- direction: 'pull'
- dry_run: false

This will restore the entire application to the previous cloud state.

### Emergency Procedures

If the application is completely unresponsive:
1. Check Fly.io dashboard for container status
2. Review recent logs for errors
3. Consider deploying a known-good version using a specific image tag

",
        );
    }

    prompt.push_str(
        r"
## Best Practices

1. **Always dry run first** - Use dry_run: true before any actual sync
2. **Monitor after deploy** - Watch logs for the first few minutes
3. **Small, frequent deploys** - Prefer smaller changes over large batches
4. **Document changes** - Keep track of what was deployed and when
5. **Test locally** - Ensure all tests pass before syncing

## Common Issues

- **Auth errors**: Check that your API token is valid and not expired
- **Sync conflicts**: If files differ unexpectedly, investigate before overwriting
- **Build failures**: Check cargo build output for compilation errors
- **Deployment timeouts**: Fly.io may need health check adjustments
",
    );

    prompt
}
