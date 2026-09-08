//! Per-tab loading for the user detail page.
//!
//! Every function here loads exactly one tab. A failed read degrades to an
//! empty panel rather than taking the page down: the header already answers
//! "who is this account", which is most of why the page was opened.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::repositories::analytics::conversation_rows::{
    ConversationFilter, ConversationRow, ConversationTotals, find_latest_conversation,
    get_conversation_totals, list_recent_conversations,
};
use crate::repositories::users::enrolment::{UserCommitRow, UserDeviceRow};
use crate::repositories::users::sessions::SigninSessionRow;

// Why: how many commits the Usage tab lists before deferring to the sessions
// pages.
pub(super) const COMMIT_LIMIT: i64 = 20;

// Why: how many sign-in sessions one page of the Sessions tab carries.
pub(super) const SESSION_PAGE_SIZE: i64 = 50;

pub(super) struct IdentityData {
    pub roles: Vec<String>,
    pub adfs_groups: Vec<String>,
    pub identities: Vec<repositories::users::federated::LinkedIdentityRow>,
    pub salesforce_username: Option<String>,
    pub share_token_version: i32,
}

pub(super) async fn load_identity(
    pool: &PgPool,
    user_id: &UserId,
    roles: &[String],
) -> IdentityData {
    let (adfs, identities, salesforce, share) = tokio::join!(
        repositories::groups::members::list_source_ad_groups(pool, user_id),
        repositories::users::federated::list_linked_identities(pool, user_id),
        repositories::users::salesforce_identity::find_username(pool, user_id),
        repositories::users::share_token::find_share_token_version(pool, user_id),
    );
    IdentityData {
        roles: roles.to_vec(),
        adfs_groups: warn_empty(adfs, "AD groups"),
        identities: warn_empty(identities, "federated identities"),
        salesforce_username: salesforce.unwrap_or_default(),
        share_token_version: share.unwrap_or_default().unwrap_or(0),
    }
}

pub(super) struct MembershipData {
    pub groups: Vec<crate::handlers::ssr::types::MembershipChoiceView>,
    pub projects: Vec<crate::handlers::ssr::types::MembershipChoiceView>,
    pub defaults: Option<repositories::scope::defaults::ScopeDefaults>,
}

pub(super) async fn load_membership(pool: &PgPool, user_id: &UserId) -> MembershipData {
    let (groups, projects, defaults) = tokio::join!(
        super::super::scope_data::group_choices(pool, user_id),
        super::super::scope_data::project_choices(pool, user_id),
        repositories::scope::defaults::find_scope_defaults(pool, user_id),
    );
    MembershipData {
        groups,
        projects,
        defaults: defaults
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: scope defaults unavailable"))
            .unwrap_or_default(),
    }
}

pub(super) struct AccessData {
    pub matrix: Option<repositories::users::access_control::UserMatrix>,
    pub rules: Vec<crate::types::access_control::AccessControlRule>,
}

// Why: the same catalogue and the same resolver the group Access tab and the
// enforcement webhook use, so a cell here is the decision the gateway makes.
// The rule list is read a second time to find this person's own rule per
// entity, which the resolved matrix reports only as "decided by user".
pub(super) async fn load_access(pool: &PgPool, user_id: &UserId) -> AccessData {
    let Ok(services_path) = crate::handlers::shared::get_services_path() else {
        tracing::warn!("services path unavailable; user access matrix skipped");
        return AccessData {
            matrix: None,
            rules: Vec::new(),
        };
    };
    let sections = crate::handlers::access_control::build_matrix_sections(&services_path);
    let (matrix, rules) = tokio::join!(
        repositories::users::access_control::resolve_user_matrix(pool, user_id, sections),
        repositories::users::access_control::list_all_rules(pool),
    );
    AccessData {
        matrix: matrix
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: access matrix failed"))
            .ok()
            .flatten(),
        rules: warn_empty(rules, "access rules"),
    }
}

pub(super) async fn load_devices(pool: &PgPool, user_id: &UserId) -> Vec<UserDeviceRow> {
    warn_empty(
        repositories::users::enrolment::list_user_devices(pool, user_id).await,
        "devices",
    )
}

pub(super) async fn load_sessions(pool: &PgPool, user_id: &UserId) -> Vec<SigninSessionRow> {
    warn_empty(
        repositories::users::sessions::list_signin_sessions(pool, user_id).await,
        "sign-in sessions",
    )
}

pub(super) struct UsageData {
    pub summary: repositories::users::usage::UserGatewayUsage,
    pub models: Vec<repositories::users::usage::ModelShare>,
    pub latest: Option<ConversationRow>,
    pub recent: Vec<ConversationRow>,
    pub totals: ConversationTotals,
    pub commits: Vec<UserCommitRow>,
}

// Why: how many recent conversations the Usage tab lists before deferring to
// the conversations page, which carries the full list with its own paging.
const RECENT_CONVERSATIONS: i64 = 10;

// Why: no project filter — this is the admin's view of one named account, so
// the project scoping that guards the cross-user list has nothing to add and
// would only hide rows from an admin who asked for them by id.
pub(super) async fn load_usage(pool: &PgPool, user_id: &UserId) -> UsageData {
    let filter = ConversationFilter {
        user_id: Some(user_id.clone()),
        include_side_calls: true,
        ..ConversationFilter::default()
    };
    let (summary, models, latest, recent, totals, commits) = tokio::join!(
        repositories::users::usage::get_ai_request_summary(pool, user_id),
        repositories::users::usage::list_top_models(pool, user_id, None, 10),
        find_latest_conversation(pool, user_id),
        list_recent_conversations(pool, user_id, RECENT_CONVERSATIONS),
        get_conversation_totals(pool, &filter),
        repositories::users::enrolment::list_user_commits(pool, user_id, COMMIT_LIMIT),
    );
    UsageData {
        summary: summary
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: usage summary failed"))
            .unwrap_or_default(),
        models: warn_empty(models, "top models"),
        latest: latest
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: latest conversation failed"))
            .unwrap_or_default(),
        recent: warn_empty(recent, "recent conversations"),
        totals: totals
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: conversation totals failed"))
            .unwrap_or_default(),
        commits: warn_empty(commits, "commits"),
    }
}

fn warn_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.inspect_err(|e| tracing::warn!(error = %e, panel = what, "user detail: panel unavailable"))
        .unwrap_or_default()
}
