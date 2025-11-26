use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Blog module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        CreateContent => Some(include_str!(
            "../../../../blog/src/queries/core/content/create_content.sql"
        )),
        GetContentByUrl => Some(include_str!(
            "../../../../blog/src/queries/core/content/get_content_by_slug.sql"
        )),
        GetContentBySourceAndSlug => Some(include_str!(
            "../../../../blog/src/queries/core/content/get_content_by_source_and_slug.sql"
        )),
        ListContent => Some(include_str!(
            "../../../../blog/src/queries/core/content/list_content.sql"
        )),
        ListAllContent => Some(include_str!(
            "../../../../blog/src/queries/core/content/list_all_content.sql"
        )),
        ListContentBySource => Some(include_str!(
            "../../../../blog/src/queries/core/content/list_content_by_source.sql"
        )),
        GetContentById => Some(include_str!(
            "../../../../blog/src/queries/core/content/get_content_by_id.sql"
        )),
        GetSocialContentByParent => Some(include_str!(
            "../../../../blog/src/queries/core/content/get_social_content_by_parent.sql"
        )),
        UpdateContent => Some(include_str!(
            "../../../../blog/src/queries/core/content/update_content.sql"
        )),
        DeleteContent => Some(include_str!(
            "../../../../blog/src/queries/core/content/delete_content.sql"
        )),
        DeleteContentBySource => Some(include_str!(
            "../../../../blog/src/queries/core/content/delete_content_by_source.sql"
        )),
        AddLinksColumnToContent => Some(include_str!(
            "../../../../blog/schema/009_add_links_column.sql"
        )),
        SearchByCategory => Some(include_str!(
            "../../../../blog/src/queries/core/search/search_by_category.sql"
        )),
        SearchByTags => Some(include_str!(
            "../../../../blog/src/queries/core/search/search_by_tags.sql"
        )),
        SearchContentByKeyword => Some(include_str!(
            "../../../../blog/src/queries/core/search/search_by_keyword.sql"
        )),
        CreateTag => Some(include_str!(
            "../../../../blog/src/queries/core/tags/create_tag.sql"
        )),
        GetTagByName => Some(include_str!(
            "../../../../blog/src/queries/core/tags/get_tag_by_name.sql"
        )),
        ListTags => Some(include_str!(
            "../../../../blog/src/queries/core/tags/list_tags.sql"
        )),
        LinkTagToContent => Some(include_str!(
            "../../../../blog/src/queries/core/tags/link_tag_to_content.sql"
        )),
        UnlinkAllTagsFromContent => Some(include_str!(
            "../../../../blog/src/queries/core/tags/unlink_all_tags_from_content.sql"
        )),
        GetTagsByContent => Some(include_str!(
            "../../../../blog/src/queries/core/tags/get_tags_by_content.sql"
        )),
        CreateLink => Some(include_str!(
            "../../../../blog/src/queries/core/links/create_link.sql"
        )),
        GetLinkById => Some(include_str!(
            "../../../../blog/src/queries/core/links/get_link_by_id.sql"
        )),
        GetLinkByShortCode => Some(include_str!(
            "../../../../blog/src/queries/core/links/get_link_by_short_code.sql"
        )),
        ListLinksByCampaign => Some(include_str!(
            "../../../../blog/src/queries/core/links/list_links_by_campaign.sql"
        )),
        ListLinksBySourceContent => Some(include_str!(
            "../../../../blog/src/queries/core/links/list_links_by_source_content.sql"
        )),
        IncrementLinkClicks => Some(include_str!(
            "../../../../blog/src/queries/core/links/increment_link_clicks.sql"
        )),
        RecordClick => Some(include_str!(
            "../../../../blog/src/queries/core/links/record_click.sql"
        )),
        GetClicksByLink => Some(include_str!(
            "../../../../blog/src/queries/core/links/get_clicks_by_link.sql"
        )),
        CheckSessionClickedLink => Some(include_str!(
            "../../../../blog/src/queries/core/links/check_session_clicked_link.sql"
        )),
        GetLinkPerformance => Some(include_str!(
            "../../../../blog/src/queries/core/links/get_link_performance.sql"
        )),
        GetAggregatedLinkPerformance => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_link_performance.sql"
        )),
        GetCampaignPerformance => Some(include_str!(
            "../../../../blog/src/queries/core/links/get_campaign_performance.sql"
        )),
        GetContentJourneyMap => Some(include_str!(
            "../../../../blog/src/queries/core/links/get_content_journey_map.sql"
        )),
        GetTopContent => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_top_content.sql"
        )),
        GetCategoryPerformance => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_category_performance.sql"
        )),
        GetContentTrends => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_content_trends.sql"
        )),
        GetDailyViewsPerContent => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_daily_views_per_content.sql"
        )),
        GetTrafficSources => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_traffic_sources.sql"
        )),
        GetTopReferrers => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_top_referrers.sql"
        )),
        GetDeviceLocation => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/get_device_location.sql"
        )),
        GetContentClickMetrics => Some(include_str!(
            "../../../../blog/src/queries/core/analytics/postgres/get_content_click_metrics.sql"
        )),
        UpdateContentImage => Some(include_str!(
            "../../../../blog/src/queries/core/content/update_content_image.sql"
        )),

        _ => None,
    }
}
