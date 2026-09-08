//! The shell every admin page renders inside: branding, the signed-in user,
//! marketplace standing, and the per-page help strip, wrapped around the
//! page's own context.

use serde::Serialize;
use systemprompt_web_shared::{BrandingConfig, RankTier, UserId};

use crate::numeric;
use crate::types::{MarketplaceContext, UserContext};

use super::ssr_demo_help::demo_help_text;

#[derive(Debug, Serialize)]
pub(crate) struct CurrentUser<'a> {
    user_id: &'a UserId,
    username: &'a str,
    roles: &'a [String],
    is_admin: bool,
    // Why: the sidebar gates read sections on `is_console` and write controls
    // on `is_admin`, so a project manager sees the People pages without
    // seeing the buttons that change them.
    is_console: bool,
    is_platform_admin: bool,
}

impl<'a> From<&'a UserContext> for CurrentUser<'a> {
    fn from(ctx: &'a UserContext) -> Self {
        Self {
            user_id: &ctx.user_id,
            username: &ctx.username,
            roles: &ctx.roles,
            is_admin: ctx.is_admin,
            is_console: ctx.is_console,
            is_platform_admin: ctx.is_platform_admin,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct MarketplaceView<'a> {
    user_id: &'a UserId,
    site_url: &'a str,
    total_plugins: usize,
    total_skills: usize,
    agents_count: usize,
    mcp_count: usize,
    rank_level: i32,
    rank_name: &'a str,
    rank_tier: RankTier,
    total_xp: i64,
    xp_progress_pct: i64,
    has_completed_onboarding: bool,
    current_streak: i64,
    longest_streak: i64,
    next_rank_name: &'a str,
    xp_to_next_rank: i64,
}

impl<'a> From<&'a MarketplaceContext> for MarketplaceView<'a> {
    fn from(ctx: &'a MarketplaceContext) -> Self {
        Self {
            user_id: &ctx.user_id,
            site_url: &ctx.site_url,
            total_plugins: ctx.total_plugins,
            total_skills: ctx.total_skills,
            agents_count: ctx.agents_count,
            mcp_count: ctx.mcp_count,
            rank_level: ctx.rank_level,
            rank_name: &ctx.rank_name,
            rank_tier: ctx.rank_tier,
            total_xp: ctx.total_xp,
            xp_progress_pct: numeric::round_to_i64(ctx.xp_progress_pct),
            has_completed_onboarding: ctx.has_completed_onboarding,
            current_streak: ctx.current_streak,
            longest_streak: ctx.longest_streak,
            next_rank_name: &ctx.next_rank_name,
            xp_to_next_rank: ctx.xp_to_next_rank,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct BrandingShell<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) branding: Option<&'a BrandingConfig>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PageShell<'a, T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    branding: Option<&'a BrandingConfig>,
    current_user: CurrentUser<'a>,
    marketplace: MarketplaceView<'a>,
    page_stats: [(); 0],
    #[serde(skip_serializing_if = "Option::is_none")]
    demo_help: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    demo_help_url: Option<String>,
    // Why: the shell decides this, not the page. Every AI-activity page shares
    // one scope contract, and a page author who forgot the flag would silently
    // lose the control — so it is derived from the page id in one place.
    scope_selector: bool,
    #[serde(flatten)]
    page: &'a T,
}

impl<'a, T: Serialize> PageShell<'a, T> {
    pub(crate) fn new(
        branding: Option<&'a BrandingConfig>,
        user_ctx: &'a UserContext,
        mkt_ctx: &'a MarketplaceContext,
        page_id: Option<&str>,
        page: &'a T,
    ) -> Self {
        let help = page_id.map(demo_help_text);
        Self {
            branding,
            current_user: CurrentUser::from(user_ctx),
            marketplace: MarketplaceView::from(mkt_ctx),
            page_stats: [],
            demo_help: help.map(|(text, _)| text),
            demo_help_url: help.map(|(_, slug)| format!("/documentation/{slug}")),
            scope_selector: takes_scope_selector(page_id),
            page,
        }
    }
}

// Why: the AI-activity listings, the pages whose rows a scope and a time range
// narrow. Detail pages are absent on purpose: one row is already the narrowest
// scope there is, and a filter above it would do nothing.
const SCOPED_PAGES: [&str; 5] = [
    "analytics-dashboard",
    "requests",
    "sessions",
    "traces",
    "contexts",
];

fn takes_scope_selector(page_id: Option<&str>) -> bool {
    page_id.is_some_and(|id| SCOPED_PAGES.contains(&id))
}
