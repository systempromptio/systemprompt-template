use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Mcp module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        InsertToolExecution => Some(include_str!(
            "../../../../mcp/src/queries/tool_usage/postgres/start_execution.sql"
        )),
        GetToolExecution => Some(include_str!(
            "../../../../mcp/src/queries/tool_usage/postgres/get_execution_by_id.sql"
        )),
        UpdateToolExecutionResult => Some(include_str!(
            "../../../../mcp/src/queries/tool_usage/postgres/log_execution_sync.sql"
        )),
        RegisterMcpServer => Some(include_str!(
            "../../../../mcp/src/queries/core/postgres/register_service.sql"
        )),
        ListMcpServers => Some(include_str!(
            "../../../../mcp/src/queries/core/postgres/get_running_services.sql"
        )),
        UpdateMcpServerStatus => Some(include_str!(
            "../../../../mcp/src/queries/core/postgres/update_service_status.sql"
        )),
        RemoveMcpServer => Some(include_str!(
            "../../../../mcp/src/queries/core/postgres/mark_service_crashed.sql"
        )),

        _ => None,
    }
}
