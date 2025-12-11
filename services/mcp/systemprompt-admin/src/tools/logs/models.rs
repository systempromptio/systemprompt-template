#[derive(Debug)]
#[allow(dead_code)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub context_id: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LogStats {
    pub total_logs: i64,
    pub error_count: i64,
    pub warn_count: i64,
    pub info_count: i64,
    pub unique_modules: i64,
    pub unique_users: i64,
    pub last_log_time: Option<String>,
}
