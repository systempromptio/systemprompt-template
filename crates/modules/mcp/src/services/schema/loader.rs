use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct SchemaLoader;

impl SchemaLoader {
    pub fn load_schema_file(service_path: &Path, schema_file: &str) -> Result<String> {
        let schema_path = service_path.join(schema_file);

        if !schema_path.exists() {
            anyhow::bail!(
                "Schema file not found: {} (full path: {})",
                schema_file,
                schema_path.display()
            );
        }

        let content = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

        if content.trim().is_empty() {
            anyhow::bail!("Schema file is empty: {schema_file}");
        }

        Ok(content)
    }

    pub fn list_schema_files(service_path: &Path) -> Result<Vec<PathBuf>> {
        let schema_dir = service_path.join("schema");

        if !schema_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&schema_dir).with_context(|| {
            format!("Failed to read schema directory: {}", schema_dir.display())
        })?;

        let mut schema_files = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                schema_files.push(path);
            }
        }

        Ok(schema_files)
    }

    pub fn validate_schema_syntax(sql: &str) -> Result<()> {
        let sql_upper = sql.trim().to_uppercase();

        if !sql_upper.starts_with("CREATE TABLE") && !sql_upper.starts_with("--") {
            anyhow::bail!("Schema must start with CREATE TABLE statement");
        }

        if !sql_upper.contains("CREATE TABLE") {
            anyhow::bail!("Schema must contain at least one CREATE TABLE statement");
        }

        Ok(())
    }

    pub fn validate_table_naming(sql: &str, module_name: &str) -> Result<()> {
        let module_prefix = module_name.replace('-', "_");

        let table_names = Self::extract_table_names(sql)?;

        if table_names.is_empty() {
            anyhow::bail!("No CREATE TABLE statements found in schema");
        }

        for table_name in &table_names {
            if !table_name.starts_with(&module_prefix) {
                anyhow::bail!(
                    "Table name '{table_name}' must start with module prefix '{module_prefix}' (from module '{module_name}')"
                );
            }
        }

        Ok(())
    }

    fn extract_table_names(sql: &str) -> Result<Vec<String>> {
        let sql_upper = sql.to_uppercase();
        let mut table_names = Vec::new();

        let lines: Vec<&str> = sql_upper.lines().collect();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("CREATE TABLE") || trimmed.contains("CREATE TABLE IF NOT EXISTS")
            {
                if let Some(table_name) = Self::parse_table_name(trimmed) {
                    table_names.push(table_name);
                }
            }
        }

        Ok(table_names)
    }

    fn parse_table_name(line: &str) -> Option<String> {
        let line = line.trim();

        let start_patterns = ["CREATE TABLE IF NOT EXISTS ", "CREATE TABLE "];

        for pattern in &start_patterns {
            if let Some(after_create) = line.strip_prefix(pattern) {
                let table_name = after_create
                    .split_whitespace()
                    .next()?
                    .trim_matches('(')
                    .trim_matches('"')
                    .trim_matches('`')
                    .to_string();

                return Some(table_name);
            }
        }

        None
    }
}
