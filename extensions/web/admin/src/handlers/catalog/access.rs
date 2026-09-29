//! The "Who gets this" panel as the catalog detail pages build it: the
//! entity's own page URL, so the "Why?" form and every access link stay on
//! the page the reader is already looking at.

use sqlx::PgPool;

use super::view;
use crate::handlers::ssr::entity_panel::{EntityAccessView, PanelRequest, build_entity_panel};
use crate::types::{ENTITY_PLUGIN, ENTITY_SKILL, UserContext};

pub(crate) async fn panel(
    pool: &PgPool,
    user_ctx: &UserContext,
    entity: (&str, &str),
    why: Option<&str>,
) -> EntityAccessView {
    let page_url = match entity.0 {
        ENTITY_PLUGIN => view::plugin_url(entity.1),
        ENTITY_SKILL => view::skill_url(entity.1),
        _ => view::mcp_url(entity.1),
    };
    build_entity_panel(
        pool,
        PanelRequest {
            entity_type: entity.0,
            entity_id: entity.1,
            page_url: &page_url,
            why,
            can_write: user_ctx.is_admin,
            note: None,
        },
    )
    .await
}
