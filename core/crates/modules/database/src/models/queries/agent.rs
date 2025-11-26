use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Agent module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use, clippy::match_same_arms)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        SaveAgentCapabilities => Some(include_str!(
            "../../../../agent/src/queries/core/capabilities/postgres/save_agent_capabilities.sql"
        )),
        GetAgentCapabilities => Some(include_str!(
            "../../../../agent/src/queries/core/capabilities/postgres/get_agent_capabilities.sql"
        )),
        RegisterAgent => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/register_agent.sql"
        )),
        MarkAgentStopped => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/mark_agent_stopped.sql"
        )),
        MarkAgentCrashed => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/mark_agent_crashed.sql"
        )),
        MarkAgentError => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/mark_agent_error.sql"
        )),
        RemoveAgentService => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/remove_agent_service.sql"
        )),
        ListRunningAgents => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/list_running_agents.sql"
        )),
        GetAgentStatus => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/get_agent_status.sql"
        )),
        UpdateAgentHealth => Some(include_str!(
            "../../../../agent/src/queries/core/agent_services/postgres/update_agent_health.sql"
        )),
        InsertArtifact => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/insert_artifact.sql"
        )),
        InsertArtifactPart => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/insert_artifact_part.sql"
        )),
        GetArtifactById => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/get_artifact_by_id.sql"
        )),
        GetArtifactsByContext => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/get_artifacts_by_context.sql"
        )),
        GetArtifactsByUser => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/get_artifacts_by_user.sql"
        )),
        GetArtifactsByUserLimited => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/get_artifacts_by_user_limited.sql"
        )),
        GetArtifactsByTask => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/get_artifacts_by_task.sql"
        )),
        GetArtifactParts => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/get_artifact_parts.sql"
        )),
        LinkArtifactToContext => Some(include_str!(
            "../../../../agent/src/queries/core/artifacts/postgres/link_artifact_to_context.sql"
        )),
        InsertContext => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/create_context.sql"
        )),
        GetContext => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/get_context.sql"
        )),
        GetContextsByUser => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/list_contexts_basic.sql"
        )),
        UpdateContext => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/update_context_name.sql"
        )),
        DeleteContext => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/delete_context.sql"
        )),
        SearchContexts => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/list_contexts_with_stats.sql"
        )),
        GetLastAgentForContext => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/get_last_agent_for_context.sql"
        )),
        GetNewToolExecutionsSince => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/get_new_tool_executions_since.sql"
        )),
        GetNewTaskStatusChangesSince => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/get_new_task_status_changes_since.sql"
        )),
        GetNewArtifactsSince => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/get_new_artifacts_since.sql"
        )),
        GetContextUpdatesSince => Some(include_str!(
            "../../../../agent/src/queries/contexts/postgres/get_context_updates_since.sql"
        )),
        TrackAgentInContext => Some(include_str!(
            "../../../../agent/src/queries/context_agents/postgres/track_agent_in_context.sql"
        )),
        InsertMessage => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/insert_message.sql"
        )),
        InsertMessagePartText => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/insert_message_part_text.sql"
        )),
        InsertMessagePartFile => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/insert_message_part_file.sql"
        )),
        InsertMessagePartData => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/insert_message_part_data.sql"
        )),
        GetMessageParts => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_message_parts.sql"
        )),
        GetMessagePartsByTask => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_message_parts_by_task.sql"
        )),
        InsertTask => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/insert_task.sql"
        )),
        GetTask => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_task.sql"
        )),
        ListTasksByContext => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/list_tasks_by_context.sql"
        )),
        GetTasksByUser => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_tasks_by_user.sql"
        )),
        GetTasksByUserPaged => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_tasks_by_user_paged.sql"
        )),
        InsertTaskSimple => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/insert_task_simple.sql"
        )),
        UpdateTaskStatus => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/update_task_status.sql"
        )),
        UpdateTaskStatusCompleted => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/update_task_status_completed.sql"
        )),
        UpdateTaskWithMetadata => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/update_task_with_metadata.sql"
        )),
        GetMaxSequenceNumber => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_max_sequence_number.sql"
        )),
        GetTaskMessages => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_task_messages.sql"
        )),
        GetContextMessages => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_context_messages.sql"
        )),
        DeleteMessageParts => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/delete_message_parts.sql"
        )),
        DeleteTaskMessage => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/delete_task_message.sql"
        )),
        InsertPushNotificationConfig => Some(include_str!(
            "../../../../agent/src/queries/notifications/postgres/insert_push_notification_config.sql"
        )),
        GetPushNotificationConfigById => Some(include_str!(
            "../../../../agent/src/queries/notifications/postgres/get_push_notification_config_by_id.sql"
        )),
        DeletePushNotificationConfig => Some(include_str!(
            "../../../../agent/src/queries/notifications/postgres/delete_push_notification_configs_by_task.sql"
        )),
        DeletePushNotificationConfigById => Some(include_str!(
            "../../../../agent/src/queries/notifications/postgres/delete_push_notification_config_by_id.sql"
        )),
        GetUserByContext => Some(include_str!(
            "../../../../agent/src/queries/contexts/notifications/postgres/get_user_by_context.sql"
        )),
        InsertContextNotification => Some(include_str!(
            "../../../../agent/src/queries/contexts/notifications/postgres/insert_notification.sql"
        )),
        MarkNotificationBroadcasted => Some(include_str!(
            "../../../../agent/src/queries/contexts/notifications/postgres/mark_broadcasted.sql"
        )),
        GetMessageById => Some(include_str!(
            "../../../../agent/src/queries/contexts/webhooks/postgres/get_message_by_id.sql"
        )),
        GetTaskContextUser => Some(include_str!(
            "../../../../agent/src/queries/tasks/postgres/get_context_user.sql"
        )),
        GetAgentConversationStats => Some(include_str!(
            "../../../../agent/src/queries/analytics/postgres/get_agent_conversation_stats.sql"
        )),
        GetTopAgentsByConversations => Some(include_str!(
            "../../../../agent/src/queries/analytics/postgres/get_top_agents_by_conversations.sql"
        )),
        CreateSkill => Some(include_str!(
            "../../../../agent/src/queries/skills/postgres/create_skill.sql"
        )),
        GetSkillById => Some(include_str!(
            "../../../../agent/src/queries/skills/postgres/get_skill_by_id.sql"
        )),
        GetSkillByFilePath => Some(include_str!(
            "../../../../agent/src/queries/skills/postgres/get_skill_by_file_path.sql"
        )),
        ListEnabledSkills => Some(include_str!(
            "../../../../agent/src/queries/skills/postgres/list_enabled_skills.sql"
        )),
        ListAllSkills => Some(include_str!(
            "../../../../agent/src/queries/skills/postgres/list_all_skills.sql"
        )),
        UpdateSkill => Some(include_str!(
            "../../../../agent/src/queries/skills/postgres/update_skill.sql"
        )),

        _ => None,
    }
}
