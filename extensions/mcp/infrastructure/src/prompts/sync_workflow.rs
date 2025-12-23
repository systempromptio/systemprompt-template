#[must_use]
pub fn build_sync_workflow_prompt(direction: &str, scope: &str) -> String {
    let direction_desc = match direction {
        "push" => "uploading local changes to cloud",
        "pull" => "downloading cloud state to local",
        _ => "syncing between local and cloud",
    };

    let scope_desc = match scope {
        "files" => "configuration files only",
        "database" => "database records only",
        "all" => "files, database, and deployment",
        _ => "all resources",
    };

    format!(
        r"# SystemPrompt Sync Workflow

## Current Operation: {direction} - {scope}

You are {direction_desc}, targeting {scope_desc}.

## Workflow Steps

### 1. Pre-Sync Checks

Before syncing, always:
- Run `sync_status` to verify connectivity
- Ensure you have the latest local changes committed
- Check for any pending migrations

### 2. Dry Run Phase

**Critical**: Always perform a dry run first!

```
Use the appropriate sync tool with dry_run: true
```

Review the output carefully:
- Files/records that will be created
- Files/records that will be updated
- Files/records that will be deleted
- Any skipped items and reasons

### 3. Review Changes

For each change type:

**Created items**: New content being added
- Verify these are intentional additions
- Check for any sensitive data

**Updated items**: Modified content
- Confirm the changes are expected
- Review any merge conflicts

**Deleted items**: Content being removed
- Ensure nothing critical is being removed
- Consider backup if needed

### 4. Execute Sync

Once satisfied with the dry run results:

```
Run the sync with dry_run: false
```

Monitor the output for:
- Successful operations
- Any errors or warnings
- Final summary statistics

### 5. Post-Sync Verification

After sync completes:
1. Run `sync_status` again to confirm state
2. Verify the changes in the target environment
3. Test affected functionality

## Scope-Specific Guidelines

{scope_guidelines}

## Error Handling

If errors occur during sync:

1. **Note the error message** - It will indicate what failed
2. **Check connectivity** - Network issues are common
3. **Verify credentials** - API tokens may have expired
4. **Review permissions** - Ensure proper access rights
5. **Retry with smaller scope** - Sync specific tables/files if needed

## Best Practices

- **Consistent direction**: Complete a full sync in one direction before switching
- **Regular syncs**: Smaller, frequent syncs are easier to manage
- **Backup first**: For production data, ensure backups exist
- **Monitor changes**: Keep track of what's being synced and why
",
        direction = direction,
        scope = scope,
        direction_desc = direction_desc,
        scope_desc = scope_desc,
        scope_guidelines = get_scope_guidelines(scope)
    )
}

fn get_scope_guidelines(scope: &str) -> &'static str {
    match scope {
        "files" => {
            r"### File Sync Guidelines

Files included:
- Agent configuration files (*.yml, *.yaml)
- Skill definitions
- Content templates
- Web configuration files

Files excluded:
- Secrets and credentials
- Build artifacts
- Node modules / target directories
- Local development overrides

Tips:
- Check file permissions after sync
- Verify YAML syntax is valid
- Watch for environment-specific values"
        }
        "database" => {
            r"### Database Sync Guidelines

Tables synced:
- agents: Agent definitions and configurations
- skills: Skill metadata and parameters
- contexts: User context templates

Considerations:
- UUIDs must remain consistent across environments
- Relationships between tables are maintained
- Timestamps are preserved from source
- Soft-deleted records are handled appropriately

Tips:
- Consider syncing specific tables for targeted updates
- Watch for foreign key dependencies
- Verify data integrity after sync"
        }
        "all" => {
            r"### Full Sync Guidelines

This syncs everything:
1. Configuration files
2. Database records
3. Builds and deploys the application (push only)

Order of operations:
1. Files are synced first
2. Database records are synced second
3. Application is built and deployed (if pushing)

Tips:
- Allow sufficient time for full sync
- Monitor each phase for errors
- Have rollback plan ready
- Test thoroughly after completion"
        }
        _ => "Follow standard sync procedures for your use case.",
    }
}
