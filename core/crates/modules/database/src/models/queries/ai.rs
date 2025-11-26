use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Ai module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        InsertAiRequest => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/insert_ai_request.sql"
        )),
        InsertAiImageRequest => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/insert_ai_image_request.sql"
        )),
        GetAiRequest => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_ai_request_by_id.sql"
        )),
        ListAiRequestsBySession => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_session_ai_usage.sql"
        )),
        ListAiRequestsByUser => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_user_ai_usage_base.sql"
        )),
        UpdateAiRequestStatus => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/update_ai_request.sql"
        )),
        InsertRequestMessage => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/insert_message.sql"
        )),
        InsertResponseMessage => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/insert_response_message.sql"
        )),
        GetAiMessageMaxSequence => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_max_sequence.sql"
        )),
        GetRequestMessages => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_messages.sql"
        )),
        InsertToolCall => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/insert_tool_call.sql"
        )),
        GetToolCalls => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_tool_calls.sql"
        )),
        GetTokenUsageByModel => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_cost_summary_by_user.sql"
        )),
        GetUserAiUsageAll => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_user_ai_usage_all.sql"
        )),
        GetUserAiUsageWithDateRange => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_user_ai_usage_with_date_range.sql"
        )),
        GetUserAiUsageSinceDate => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_user_ai_usage_since_date.sql"
        )),
        GetUserAiUsageUntilDate => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_user_ai_usage_until_date.sql"
        )),
        GetProviderUsageAll => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_provider_usage_all.sql"
        )),
        GetProviderUsageByUser => Some(include_str!(
            "../../../../ai/src/queries/ai/postgres/get_provider_usage_by_user.sql"
        )),
        InsertGeneratedImage => Some(include_str!(
            "../../../../ai/src/queries/postgres/insert_generated_image.sql"
        )),
        GetGeneratedImageByUuid => Some(include_str!(
            "../../../../ai/src/queries/postgres/get_generated_image_by_uuid.sql"
        )),
        ListGeneratedImagesByUser => Some(include_str!(
            "../../../../ai/src/queries/postgres/list_generated_images_by_user.sql"
        )),
        DeleteGeneratedImage => Some(include_str!(
            "../../../../ai/src/queries/postgres/delete_generated_image.sql"
        )),

        _ => None,
    }
}
