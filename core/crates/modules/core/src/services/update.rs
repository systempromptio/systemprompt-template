use crate::models::modules::Module;
use anyhow::Result;
use systemprompt_core_database::{Database, DatabaseProvider};
use systemprompt_core_logging::LogService;
use systemprompt_models::{Config, SystemPaths};

pub async fn update_module(module: &Module) -> Result<()> {
    let app_context = crate::AppContext::new().await.unwrap();
    let db = app_context.db_pool().clone();
    let log = LogService::system(db.clone());

    let _ = log
        .info(
            "core_update",
            &format!(
                "Updating module: {} version: {}",
                module.name, module.version
            ),
        )
        .await;

    let current_version = get_current_version(module, db.as_ref()).await?;

    if current_version != module.version {
        run_migrations(module, &current_version, db.as_ref(), &log).await?;
        apply_schema_updates(module, db.as_ref()).await?;
    }

    let _ = log
        .info(
            "core_update",
            &format!(
                "Module updated successfully: {} version: {}",
                module.name, module.version
            ),
        )
        .await;

    Ok(())
}

async fn run_migrations(
    module: &Module,
    from_version: &str,
    db: &Database,
    log: &LogService,
) -> Result<()> {
    let config = Config::global();
    let migrations_path = SystemPaths::core_migrations(config, &module.name);

    if !migrations_path.exists() {
        return Ok(());
    }

    let mut migrations = Vec::new();
    for entry in std::fs::read_dir(migrations_path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if entry.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("sql")) {
            if let Some(version) = extract_migration_version(&file_name) {
                if version > from_version && version <= module.version.as_str() {
                    migrations.push((version.to_string(), entry.path()));
                }
            }
        }
    }

    migrations.sort_by(|a, b| a.0.cmp(&b.0));

    for (version, path) in migrations {
        let _ = log
            .info(
                "core_update",
                &format!("Applying migration: {}", path.display()),
            )
            .await;
        let sql = std::fs::read_to_string(&path)?;

        db.execute_batch(&sql)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {e}"))?;

        let _ = log
            .info(
                "core_update",
                &format!("Migration completed: version {version}"),
            )
            .await;
    }

    Ok(())
}

fn extract_migration_version(filename: &str) -> Option<&str> {
    filename.strip_suffix(".sql")
}

async fn get_current_version(module: &Module, db: &Database) -> Result<String> {
    let query = "SELECT version FROM modules WHERE name = ?";
    let result = db.fetch_optional(&query, &[&module.name.as_str()]).await?;

    match result {
        Some(row) => {
            let version = row
                .get("version")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Version field missing for module {}", module.name))?
                .to_string();
            Ok(version)
        },
        None => Err(anyhow::anyhow!(
            "Module {} not found in database",
            module.name
        )),
    }
}

async fn apply_schema_updates(module: &Module, db: &Database) -> Result<()> {
    if let Some(schemas) = &module.schemas {
        for schema in schemas {
            let schema_path = format!("crates/core/{}/src/{}", module.name, schema.file);
            if let Ok(sql) = std::fs::read_to_string(&schema_path) {
                db.execute_batch(&sql).await?;
            }
        }
    }

    Ok(())
}
