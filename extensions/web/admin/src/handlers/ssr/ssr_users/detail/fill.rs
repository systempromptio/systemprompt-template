//! Loading the one tab the reader asked for on `/admin/users/{id}`.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::context::UserDetailContext;
use crate::env::AdminEnv;
#[cfg(feature = "agents")]
use crate::types::UserContext;

// Why: only the active tab is loaded. Rendering all seven would make opening a
// person's Identity tab pay for their whole request history.
pub(super) struct TabRead<'a> {
    pub(super) pool: &'a PgPool,
    pub(super) user_id: &'a UserId,
    pub(super) detail: &'a crate::types::UserDetail,
    pub(super) tab: &'static str,
    pub(super) page: i64,
    #[cfg(feature = "agents")]
    pub(super) viewer: &'a UserContext,
}

pub(super) async fn fill_tab(
    env: &AdminEnv,
    read: TabRead<'_>,
    mut data: UserDetailContext,
) -> UserDetailContext {
    let TabRead {
        pool,
        user_id,
        detail,
        tab,
        page,
        #[cfg(feature = "agents")]
        viewer,
    } = read;
    match tab {
        #[cfg(feature = "agents")]
        "conversations" => {
            data.conversations = Some(super::view::conversations_tab(
                &super::load::load_conversations(pool, user_id, page).await,
                user_id,
                viewer,
                page,
                super::load::CONVERSATION_PAGE_SIZE,
            ));
        },
        "membership" => {
            data.membership = Some(super::view::membership_tab(
                super::load::load_membership(pool, user_id).await,
            ));
        },
        "access" => {
            data.access = Some(super::view::access_tab(
                super::load::load_access(env, pool, user_id).await,
                user_id,
            ));
        },
        "devices" => {
            let rows = super::load::load_devices(pool, user_id).await;
            let tab_view = super::view::devices_tab(rows);
            data.kpis.devices = i64::try_from(tab_view.count).unwrap_or(0);
            data.devices = Some(tab_view);
        },
        "sessions" => {
            let rows = super::load::load_sessions(pool, user_id).await;
            data.sessions = Some(super::view::sessions_tab(
                &rows,
                user_id,
                page,
                super::load::SESSION_PAGE_SIZE,
            ));
        },
        "usage" => {
            data.usage = Some(super::view::usage_tab(
                super::load::load_usage(pool, user_id).await,
                user_id,
            ));
        },
        _ => {
            let identity = super::load::load_identity(pool, user_id, &detail.roles).await;
            let choices = super::super::scope_data::role_choices(&identity.roles);
            data.identity = Some(super::view::identity_tab(detail, &identity, choices));
        },
    }
    data
}
