use anyhow::{Context, Result};
use systemprompt_core_database::DatabaseProvider;

#[derive(Debug, Copy, Clone)]
pub struct SqlExecutor;

impl SqlExecutor {
    pub async fn execute_statements(db: &dyn DatabaseProvider, sql: &str) -> Result<()> {
        let statements = Self::parse_sql_statements(sql);

        for statement in statements {
            Self::execute_statement(db, &statement)
                .await
                .with_context(|| format!("Failed to execute SQL statement: {statement}"))?;
        }

        Ok(())
    }

    pub fn parse_sql_statements(sql: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_trigger = false;
        let mut in_dollar_quote = false;
        let mut dollar_count = 0;

        for line in sql.lines() {
            let trimmed = line.trim();

            if Self::should_skip_line(trimmed) {
                continue;
            }

            current_statement.push_str(line);
            current_statement.push('\n');

            // Track dollar-quoted strings (PostgreSQL function bodies)
            if trimmed.contains("$$") {
                dollar_count += trimmed.matches("$$").count();
                in_dollar_quote = dollar_count % 2 == 1;
            }

            if trimmed.starts_with("CREATE TRIGGER")
                || trimmed.starts_with("CREATE OR REPLACE FUNCTION")
            {
                in_trigger = true;
            }

            if Self::is_statement_complete(trimmed, in_trigger, in_dollar_quote) {
                let stmt = current_statement.trim().to_string();
                if !stmt.is_empty() {
                    statements.push(stmt);
                }
                current_statement.clear();
                in_trigger = false;
                dollar_count = 0;
            }
        }

        // Handle any remaining statement
        let stmt = current_statement.trim().to_string();
        if !stmt.is_empty() {
            statements.push(stmt);
        }

        statements
    }

    async fn execute_statement(db: &dyn DatabaseProvider, statement: &str) -> Result<()> {
        // Use raw execution for DDL operations to avoid PostgreSQL protocol issues
        // execute_raw() uses simple query protocol (PostgreSQL) which properly
        // handles IF EXISTS/IF NOT EXISTS conditions at parse-time
        db.execute_raw(statement)
            .await
            .with_context(|| format!("Failed to execute SQL statement: {statement}"))?;

        Ok(())
    }

    fn should_skip_line(line: &str) -> bool {
        line.starts_with("--") || line.is_empty()
    }

    fn is_statement_complete(line: &str, in_trigger: bool, in_dollar_quote: bool) -> bool {
        // Don't end statement if we're inside a dollar-quoted string
        if in_dollar_quote {
            return false;
        }

        // For triggers/functions, wait for the final END; or LANGUAGE plpgsql;
        if in_trigger {
            return line == "END;" || line.ends_with("LANGUAGE plpgsql;");
        }

        // Normal statement: ends with semicolon
        line.ends_with(';')
    }
}
