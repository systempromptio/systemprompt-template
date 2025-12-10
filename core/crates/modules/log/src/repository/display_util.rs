use super::LoggingRepository;

#[derive(Debug, Clone, Copy)]
pub struct DisplayUtil;

impl DisplayUtil {
    pub fn format_component_counts(counts: &[(String, usize)]) -> String {
        LoggingRepository::format_component_counts(counts)
    }

    pub fn format_server_status(status: &str) -> String {
        LoggingRepository::format_server_status(status)
    }
}
