use crate::models::{ColumnInfo, DatabaseInfo, QueryResult, TableInfo};

pub trait DatabaseCliDisplay {
    fn display_with_cli(&self);
}

impl DatabaseCliDisplay for Vec<TableInfo> {
    fn display_with_cli(&self) {
        if self.is_empty() {
            println!("No tables found");
        } else {
            println!("Tables:");
            for table in self {
                println!("  {} (rows: {})", table.name, table.row_count);
            }
        }
    }
}

impl DatabaseCliDisplay for (Vec<ColumnInfo>, i64) {
    fn display_with_cli(&self) {
        let (columns, _) = self;
        println!("Columns:");
        for col in columns {
            let default_display = col
                .default
                .as_deref()
                .map(|d| format!("DEFAULT {d}"))
                .unwrap_or_default();

            println!(
                "  {} {} {} {} {}",
                col.name,
                col.data_type,
                if col.nullable { "NULL" } else { "NOT NULL" },
                if col.primary_key { "PK" } else { "" },
                default_display
            );
        }
    }
}

impl DatabaseCliDisplay for DatabaseInfo {
    fn display_with_cli(&self) {
        println!("Database Info:");
        println!("  Path: {}", self.path);
        println!("  Version: {}", self.version);
        println!("  Tables: {}", self.tables.len());
    }
}

impl DatabaseCliDisplay for QueryResult {
    fn display_with_cli(&self) {
        if self.columns.is_empty() {
            println!("No data returned");
            return;
        }

        // Print headers
        println!("{}", self.columns.join(" | "));
        println!("{}", "-".repeat(80));

        // Print rows
        for row in &self.rows {
            let values: Vec<String> = self
                .columns
                .iter()
                .map(|col| {
                    row.get(col).map_or("NULL".to_string(), |v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => "NULL".to_string(),
                        _ => v.to_string(),
                    })
                })
                .collect();
            println!("{}", values.join(" | "));
        }

        println!(
            "\n{} rows returned in {}ms",
            self.row_count, self.execution_time_ms
        );
    }
}
