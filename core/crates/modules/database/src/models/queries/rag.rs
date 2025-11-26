use super::super::DatabaseQueryEnum;

/// Maps DatabaseQueryEnum variants for Rag module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
pub fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        CreateContent => Some(include_str!(
            "../../../../rag/src/queries/core/content/postgres/create_content.sql"
        )),
        GetContentById => Some(include_str!(
            "../../../../rag/src/queries/core/content/postgres/get_content.sql"
        )),
        GetContentByUrl => Some(include_str!(
            "../../../../rag/src/queries/core/content/postgres/get_content_by_slug.sql"
        )),
        ListContent => Some(include_str!(
            "../../../../rag/src/queries/core/content/postgres/list_content.sql"
        )),
        UpdateContent => Some(include_str!(
            "../../../../rag/src/queries/core/content/postgres/update_content.sql"
        )),
        DeleteContent => Some(include_str!(
            "../../../../rag/src/queries/core/content/postgres/delete_content.sql"
        )),
        CreateChunk => Some(include_str!(
            "../../../../rag/src/queries/core/chunks/postgres/create_chunk.sql"
        )),
        GetChunksByContent => Some(include_str!(
            "../../../../rag/src/queries/core/chunks/postgres/get_chunks_by_content.sql"
        )),
        DeleteChunksByContent => Some(include_str!(
            "../../../../rag/src/queries/core/chunks/postgres/delete_chunks_by_content.sql"
        )),
        CreateTag => Some(include_str!(
            "../../../../rag/src/queries/core/tags/postgres/create_tag.sql"
        )),
        GetTagByName => Some(include_str!(
            "../../../../rag/src/queries/core/tags/postgres/get_tag_by_name.sql"
        )),
        ListTags => Some(include_str!(
            "../../../../rag/src/queries/core/tags/postgres/list_tags.sql"
        )),
        LinkTagToContent => Some(include_str!(
            "../../../../rag/src/queries/core/tags/postgres/link_tag_to_content.sql"
        )),
        GetTagsByContent => Some(include_str!(
            "../../../../rag/src/queries/core/tags/postgres/get_tags_by_content.sql"
        )),
        CreateCategory => Some(include_str!(
            "../../../../rag/src/queries/core/categories/postgres/create_category.sql"
        )),
        GetCategoryById => Some(include_str!(
            "../../../../rag/src/queries/core/categories/postgres/get_category.sql"
        )),
        GetCategoryByName => Some(include_str!(
            "../../../../rag/src/queries/core/categories/postgres/get_category_by_name.sql"
        )),
        ListCategories => Some(include_str!(
            "../../../../rag/src/queries/core/categories/postgres/list_categories.sql"
        )),
        SearchContentByEmbedding => Some(include_str!(
            "../../../../rag/src/queries/core/search/postgres/fts_search.sql"
        )),
        SearchChunksByEmbedding => Some(include_str!(
            "../../../../rag/src/queries/core/search/postgres/hybrid_search.sql"
        )),

        _ => None,
    }
}
