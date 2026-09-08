//! View-model builders for the conversation reader: map repository rows into
//! the typed template-context structs in `context`.
//!
//! The transcript itself is built by `handlers::ssr::transcript_view`, which
//! the owner-facing page under `/admin/history` shares — this page asks it for
//! the unredacted, unstripped view an admin is entitled to.

use crate::repositories::analytics::context_detail::{
    ContextHeader, ContextKpis, ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};

use crate::types::conversation_analytics::SessionEntityLink;

use super::context::{
    BreadcrumbView, ContextDetailPageContext, ContextRequestRowView, EntityLinkView, HeaderView,
    TabLinkView,
};
use crate::handlers::ssr::conversation_header::{
    resolve_title, stats_view, status_badge, timeline_display,
};
use crate::handlers::ssr::entity_urls::{request_detail_url, session_detail_url, trace_detail_url};
use crate::handlers::ssr::format::{format_cost, local_time};
use crate::handlers::ssr::transcript_view::{
    TranscriptOptions, build_conversation, format_latency, short_id,
};

pub(super) const TAB_CONVERSATION: &str = "conversation";
pub(super) const TAB_REQUESTS: &str = "requests";
pub(super) const TAB_TOUCHED: &str = "touched";

pub(super) fn resolve_tab(requested: Option<&str>) -> &'static str {
    match requested {
        Some(TAB_REQUESTS) => TAB_REQUESTS,
        Some(TAB_TOUCHED) => TAB_TOUCHED,
        _ => TAB_CONVERSATION,
    }
}

pub(super) const fn default_kpis() -> ContextKpis {
    ContextKpis {
        request_count: 0,
        trace_count: 0,
        error_count: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_microdollars: 0,
        first_request_at: None,
        last_request_at: None,
        model: None,
        turn_count: 0,
        side_call_count: 0,
        side_call_cost_microdollars: 0,
        tool_call_count: 0,
        models: Vec::new(),
    }
}

// Why: Everything the page renders beside its header, gathered so the builder
// stays inside clippy's argument cap.
pub(super) struct DetailInputs<'a> {
    pub kpis: &'a ContextKpis,
    pub requests: &'a [ContextRequestRow],
    pub messages: &'a [ContextMessageRow],
    pub tool_calls: &'a [ContextToolCallRow],
    pub entity_links: &'a [SessionEntityLink],
    pub active_tab: &'static str,
}

pub(super) fn build_detail_data(
    header: &ContextHeader,
    inputs: &DetailInputs<'_>,
) -> ContextDetailPageContext {
    let DetailInputs {
        kpis,
        requests,
        messages,
        tool_calls,
        entity_links,
        active_tab,
    } = *inputs;
    let entity_link_views = entity_link_views(entity_links);
    let conversation =
        build_conversation(messages, tool_calls, requests, TranscriptOptions::default());
    let title = resolve_title(header, &conversation);
    let back_url = header
        .session_id
        .as_ref()
        .map_or_else(|| "/admin/contexts".to_owned(), session_detail_url);
    let back_label = header
        .session_id
        .as_ref()
        .map_or_else(|| "Conversations".to_owned(), |_| "Session".to_owned());
    ContextDetailPageContext {
        page: "context-detail",
        tabs: tab_links(header, active_tab, requests.len(), entity_link_views.len()),
        show_conversation: active_tab == TAB_CONVERSATION,
        show_requests: active_tab == TAB_REQUESTS,
        show_touched: active_tab == TAB_TOUCHED,
        header: header_view(header, kpis),
        stats: stats_view(kpis),
        status_badge: status_badge(header.hook_status.as_deref(), kpis.error_count),
        conversation,
        has_requests: !requests.is_empty(),
        request_count: requests.len(),
        requests: requests.iter().map(request_view).collect(),
        back_url,
        back_label,
        breadcrumbs: breadcrumbs(header, &title),
        has_entity_links: !entity_link_views.is_empty(),
        entity_link_count: entity_link_views.len(),
        entity_links: entity_link_views,
        title,
    }
}

fn tab_links(
    h: &ContextHeader,
    active: &str,
    request_count: usize,
    touched_count: usize,
) -> Vec<TabLinkView> {
    let base = format!(
        "/admin/contexts/{}",
        urlencoding::encode(h.context_id.as_str())
    );
    let tab = |slug: &'static str, label: &'static str, count: Option<usize>| TabLinkView {
        slug,
        label,
        href: format!("{base}?tab={slug}"),
        is_active: slug == active,
        count: count.and_then(|c| i64::try_from(c).ok()),
    };
    vec![
        tab(TAB_CONVERSATION, "Conversation", None),
        tab(TAB_REQUESTS, "Requests", Some(request_count)),
        tab(TAB_TOUCHED, "Touched", Some(touched_count)),
    ]
}

fn breadcrumbs(h: &ContextHeader, title: &str) -> Vec<BreadcrumbView> {
    let mut crumbs = vec![BreadcrumbView::link("Conversations", "/admin/contexts")];
    if let Some(session) = h.session_id.as_ref() {
        crumbs.push(BreadcrumbView::link(
            format!("Session {}", short_id(session.as_str())),
            session_detail_url(session),
        ));
    }
    crumbs.push(BreadcrumbView::current(title));
    crumbs
}

// Why: the share is measured against the busiest entity, not the total —
// a hundred touches of one file and one of another should read as a full bar
// beside an empty one, which a share-of-total would flatten.
fn entity_link_views(links: &[SessionEntityLink]) -> Vec<EntityLinkView> {
    let max = links.iter().map(|l| l.usage_count).max().unwrap_or(0);
    links
        .iter()
        .map(|l| EntityLinkView {
            entity_type: l.entity_type.clone(),
            entity_name: l.entity_name.clone(),
            usage_count: l.usage_count,
            share_pct: if max > 0 {
                (f64::from(l.usage_count) / f64::from(max) * 100.0).round()
            } else {
                0.0
            },
        })
        .collect()
}

fn header_view(h: &ContextHeader, k: &ContextKpis) -> HeaderView {
    HeaderView {
        context_id: h.context_id.clone(),
        context_id_short: short_id(h.context_id.as_str()),
        user_id: h.user_id.clone(),
        user_url: h
            .user_id
            .as_ref()
            .map(|u| format!("/admin/user?id={}", urlencoding::encode(u.as_str()))),
        display_name: h.display_name.clone(),
        session_id: h.session_id.clone(),
        session_url: h.session_id.as_ref().map(session_detail_url),
        client_session_id: h.client_session_id.clone(),
        // Why: the sessions page keys on the hooks session id, which for
        // gateway traffic is the Claude Code session uuid the context derives from.
        hooks_session_url: h
            .client_session_id
            .as_ref()
            .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        timeline: timeline_display(k.first_request_at, k.last_request_at),
        first_at: k.first_request_at.map(local_time),
        last_at: k.last_request_at.map(local_time),
    }
}

const fn kind_tone(kind: &str) -> &'static str {
    match kind.as_bytes() {
        b"probe" => "muted",
        b"utility" => "info",
        _ => "accent",
    }
}

fn request_view(r: &ContextRequestRow) -> ContextRequestRowView {
    ContextRequestRowView {
        id: r.id.clone(),
        id_short: short_id(r.id.as_str()),
        request_url: request_detail_url(&r.id),
        trace_id: r.trace_id.clone(),
        trace_id_short: r.trace_id.as_ref().map(|t| short_id(t.as_str())),
        trace_url: r.trace_id.as_ref().map(trace_detail_url),
        kind_tone: kind_tone(&r.effective_kind),
        kind: r.effective_kind.clone(),
        message_count: r.message_count,
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        latency_display: format_latency(r.latency_ms),
        cost_display: format_cost(r.cost_microdollars),
        created_at_local: local_time(r.created_at),
    }
}
