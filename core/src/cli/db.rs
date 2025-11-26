use crate::cli::config::{get_global_config, OutputFormat};
use anyhow::{anyhow, Result};
use clap::Subcommand;
use sqlx::{Column, Row};
use std::collections::HashMap;
use systemprompt_core_database::{
    ColumnInfo, DatabaseCliDisplay, DatabaseInfo, DatabaseProvider, DatabaseQueryEnum, QueryResult,
    TableInfo,
};
use systemprompt_core_logging::CliService;
use systemprompt_core_system::models::AppContext;
use systemprompt_core_users::repository::UserRepository;

#[derive(Subcommand)]
pub enum DbCommands {
    /// Execute SQL query
    Query {
        /// SQL query to execute
        sql: String,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Execute write operation
    Execute {
        /// SQL statement to execute
        sql: String,
        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// List all tables
    Tables,
    /// Describe table schema
    Describe {
        /// Table name
        table_name: String,
    },
    /// Database information
    Info,
    /// Run database migrations
    Migrate,
    /// Assign admin role to a user
    AssignAdmin {
        /// Username or email of the user
        user: String,
    },
}

struct DatabaseTool {
    ctx: AppContext,
}

impl DatabaseTool {
    async fn new() -> Result<Self> {
        Ok(Self {
            ctx: AppContext::new().await?,
        })
    }

    async fn execute_query(&self, query: &str, read_only: bool) -> Result<QueryResult> {
        let start = std::time::Instant::now();

        if read_only && !Self::is_safe_query(query) {
            return Err(anyhow!(
                "Only SELECT, WITH, EXPLAIN, and PRAGMA queries are allowed in read-only mode"
            ));
        }

        let db_pool = self.ctx.db_pool();
        let execution_time = start.elapsed().as_millis() as u64;

        let pg_pool = db_pool.get_postgres_pool_arc()?;
        let rows = sqlx::query(query).fetch_all(pg_pool.as_ref()).await?;

        let mut columns = Vec::new();
        let mut result_rows = Vec::new();

        if let Some(first_row) = rows.first() {
            columns = first_row
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect();
        }

        for row in &rows {
            let mut row_map = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                row_map.insert(
                    column.name().to_string(),
                    self.extract_value_postgres(row, i)?,
                );
            }
            result_rows.push(row_map);
        }

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count: rows.len(),
            execution_time_ms: execution_time,
        })
    }

    fn is_safe_query(query: &str) -> bool {
        let trimmed = query.trim().to_lowercase();
        let safe_starts = ["select", "with", "explain", "pragma"];
        let unsafe_ops = [
            " drop ", " delete ", " insert ", " update ", " alter ", " create ",
        ];

        safe_starts.iter().any(|s| trimmed.starts_with(s))
            && !unsafe_ops.iter().any(|op| trimmed.contains(op))
    }

    async fn list_tables(&self) -> Result<Vec<TableInfo>> {
        let db_pool = self.ctx.db_pool();
        let query = DatabaseQueryEnum::CliListTables.get(db_pool.as_ref());

        let rows = db_pool.fetch_all(&query, &[]).await?;

        let tables = rows
            .into_iter()
            .map(|row| {
                let name = row
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing name"))?
                    .to_string();
                Ok(TableInfo {
                    name,
                    row_count: 0,
                    columns: vec![],
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(tables)
    }

    async fn describe_table(&self, table_name: &str) -> Result<(Vec<ColumnInfo>, i64)> {
        if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(anyhow!("Invalid table name"));
        }

        let db_pool = self.ctx.db_pool();
        let columns_query = DatabaseQueryEnum::CliDescribeTable.get(db_pool.as_ref());

        let rows = db_pool.fetch_all(&columns_query, &[&table_name]).await?;

        let columns = rows
            .into_iter()
            .map(|row| {
                let name = row
                    .get("column_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing column_name"))?
                    .to_string();
                let data_type = row
                    .get("data_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing data_type"))?
                    .to_string();
                let nullable_str = row
                    .get("is_nullable")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing is_nullable"))?;
                let nullable = nullable_str.to_uppercase() == "YES";
                let default = row
                    .get("column_default")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Ok(ColumnInfo {
                    name,
                    data_type,
                    nullable,
                    default,
                    primary_key: false,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let pg_pool = db_pool.get_postgres_pool_arc()?;
        let count_query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let row_count: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(pg_pool.as_ref())
            .await?;

        Ok((columns, row_count))
    }

    async fn get_db_info(&self) -> Result<DatabaseInfo> {
        let db_pool = self.ctx.db_pool();
        let query = DatabaseQueryEnum::CliGetDbVersion.get(db_pool.as_ref());

        let version_value = db_pool.fetch_scalar_value(&query, &[]).await?;
        let version = match version_value {
            systemprompt_core_database::DbValue::String(s) => s,
            _ => "Unknown".to_string(),
        };

        Ok(DatabaseInfo {
            path: "postgresql://database".to_string(),
            size: 0,
            version,
            tables: vec![],
        })
    }

    fn extract_value_postgres(
        &self,
        row: &sqlx::postgres::PgRow,
        column_index: usize,
    ) -> Result<serde_json::Value> {
        if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, |dt| {
                serde_json::Value::String(dt.and_utc().to_rfc3339())
            }));
        }
        if let Ok(val) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, |dt| {
                serde_json::Value::String(dt.to_rfc3339())
            }));
        }
        if let Ok(val) = row.try_get::<Option<String>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, serde_json::Value::String));
        }
        if let Ok(val) = row.try_get::<Option<i64>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, |i| {
                serde_json::Value::Number(i.into())
            }));
        }
        if let Ok(val) = row.try_get::<Option<i32>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, |i| {
                serde_json::Value::Number(i.into())
            }));
        }
        if let Ok(val) = row.try_get::<Option<f64>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, |f| {
                serde_json::Number::from_f64(f)
                    .map_or(serde_json::Value::Null, serde_json::Value::Number)
            }));
        }
        if let Ok(val) = row.try_get::<Option<bool>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, serde_json::Value::Bool));
        }
        // Handle TEXT[] arrays (PostgreSQL native arrays)
        if let Ok(val) = row.try_get::<Option<Vec<String>>, _>(column_index) {
            return Ok(val.map_or(serde_json::Value::Null, |arr| {
                serde_json::Value::Array(arr.into_iter().map(serde_json::Value::String).collect())
            }));
        }
        // Handle JSONB types (PostgreSQL native JSON)
        if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(column_index) {
            return Ok(val.unwrap_or(serde_json::Value::Null));
        }
        Ok(serde_json::Value::Null)
    }

    fn print_result(&self, result: &QueryResult, format: &str) {
        let config = get_global_config();
        let output_format = if format == "json" {
            OutputFormat::Json
        } else if format == "yaml" {
            OutputFormat::Yaml
        } else {
            config.output_format
        };

        match output_format {
            OutputFormat::Json => CliService::json(result),
            OutputFormat::Yaml => CliService::yaml(result),
            OutputFormat::Table => result.display_with_cli(),
        }
    }
}

async fn migrate_standalone() -> Result<()> {
    use std::sync::Arc;
    use systemprompt_core_database::Database;
    use systemprompt_core_system::{
        models::modules::Modules, services::install::install_module_with_db, Config,
    };
    use systemprompt_models::config::VerbosityLevel;

    let verbosity = VerbosityLevel::resolve();

    // Load config from environment (don't use Config::init to avoid OnceLock issue)
    let config = Config::from_env()?;

    if verbosity.should_show_verbose() {
        CliService::info(&format!("System path: {}", config.system_path));
        CliService::info(&format!("Database type: {}", config.database_type));
        CliService::info(&format!("Database URL: {}", config.database_url));
    }

    // For SQLite, ensure parent directory and database file exist
    if config.database_type.eq_ignore_ascii_case("sqlite") {
        let db_path = std::path::Path::new(&config.database_url);
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        if !db_path.exists() {
            std::fs::File::create(db_path)?;
        }
    }

    // Create database with automatic type selection (SQLite or PostgreSQL)
    let database =
        Arc::new(Database::from_config(&config.database_type, &config.database_url).await?);

    // Discover modules from module.yaml files
    let modules = Modules::load(&config.system_path)?;
    let all_modules = modules.all();

    if all_modules.is_empty() {
        CliService::warning("No modules found - check SYSTEM_PATH environment variable");
        return Ok(());
    }

    if verbosity.should_show_verbose() {
        CliService::info(&format!("Installing {} modules", all_modules.len()));
        for module in all_modules {
            CliService::info(&format!("  {}", module.name));
        }
    }

    // Install each module
    let mut error_count = 0;

    for module in all_modules {
        match install_module_with_db(module, database.as_ref()).await {
            Ok(_) => {},
            Err(e) => {
                CliService::error(&format!("{} failed: {}", module.name, e));
                error_count += 1;
            },
        }
    }

    if error_count > 0 {
        return Err(anyhow!("Some modules failed to install"));
    }

    Ok(())
}

pub async fn execute(cmd: DbCommands) -> Result<()> {
    // Handle Migrate separately to avoid triggering ModuleManager via AppContext
    if matches!(cmd, DbCommands::Migrate) {
        return match migrate_standalone().await {
            Ok(()) => {
                CliService::success("Database migration completed");
                Ok(())
            },
            Err(e) => {
                CliService::error(&format!("Migration failed: {}", e));
                Err(e)
            },
        };
    }

    let db = DatabaseTool::new().await?;
    let config = get_global_config();

    match cmd {
        DbCommands::Query { sql, format } => match db.execute_query(&sql, false).await {
            Ok(result) => {
                if config.should_show_verbose() {
                    CliService::verbose(&format!(
                        "Query returned {} rows in {}ms",
                        result.row_count, result.execution_time_ms
                    ));
                }
                db.print_result(&result, &format);
            },
            Err(e) => {
                CliService::error(&format!("Query failed: {}", e));
                return Err(e);
            },
        },
        DbCommands::Execute { sql, format } => match db.execute_query(&sql, false).await {
            Ok(result) => {
                if config.should_show_verbose() {
                    CliService::verbose(&format!(
                        "Execution completed in {}ms",
                        result.execution_time_ms
                    ));
                }
                db.print_result(&result, &format);
            },
            Err(e) => {
                CliService::error(&format!("Execution failed: {}", e));
                return Err(e);
            },
        },
        DbCommands::Tables => match db.list_tables().await {
            Ok(tables) => {
                if config.is_json_output() {
                    CliService::json(&tables);
                } else {
                    tables.display_with_cli();
                }
            },
            Err(e) => {
                CliService::error(&format!("Failed to list tables: {}", e));
                return Err(e);
            },
        },
        DbCommands::Describe { table_name } => match db.describe_table(&table_name).await {
            Ok((columns, row_count)) => {
                if config.is_json_output() {
                    CliService::json(&serde_json::json!({
                        "table": table_name,
                        "row_count": row_count,
                        "columns": columns
                    }));
                } else {
                    CliService::info(&format!("Table: {} ({} rows)", table_name, row_count));
                    (columns, row_count).display_with_cli();
                }
            },
            Err(e) => {
                CliService::error(&format!("Failed to describe table: {}", e));
                return Err(e);
            },
        },
        DbCommands::Info => match db.get_db_info().await {
            Ok(info) => {
                if config.is_json_output() {
                    CliService::json(&info);
                } else {
                    info.display_with_cli();
                }
            },
            Err(e) => {
                CliService::error(&format!("Failed to get database info: {}", e));
                return Err(e);
            },
        },
        DbCommands::Migrate => unreachable!("Migrate is handled earlier"),
        DbCommands::AssignAdmin { user } => {
            let user_repo = UserRepository::new(db.ctx.db_pool().clone());

            CliService::info(&format!("Looking up user: {}", user));

            let user_record = if user.contains('@') {
                user_repo.find_by_email(&user).await?
            } else {
                user_repo.find_by_name(&user).await?
            };

            match user_record {
                Some(u) => {
                    let current_roles = u.roles.clone();

                    if current_roles.contains(&"admin".to_string()) {
                        CliService::warning(&format!("User '{}' already has admin role", u.name));
                        return Ok(());
                    }

                    let mut new_roles = current_roles;
                    if !new_roles.contains(&"admin".to_string()) {
                        new_roles.push("admin".to_string());
                    }
                    if !new_roles.contains(&"user".to_string()) {
                        new_roles.push("user".to_string());
                    }

                    user_repo.assign_roles(&u.name, &new_roles).await?;

                    CliService::success(&format!(
                        "✅ Admin role assigned to user '{}' ({})",
                        u.name, u.email
                    ));
                    CliService::info(&format!("   Roles: {:?}", new_roles));
                },
                None => {
                    CliService::error(&format!("User '{}' not found", user));
                    return Err(anyhow!("User not found"));
                },
            }
        },
    }

    Ok(())
}
