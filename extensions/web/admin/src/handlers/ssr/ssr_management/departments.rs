//! Department listing + detail view models for the management pages.
//!
//! Holds the serde page-data shapes, the in-memory search over the listing,
//! and the member-rollup arithmetic that the `management-departments` /
//! `management-department-detail` templates consume.

use serde::{Deserialize, Serialize};

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::{BreadcrumbView, SortHeaderView};
use crate::types::departments::{
    DEFAULT_DEPARTMENT, Department, DepartmentMember, DepartmentSummary, DepartmentTopTool,
};

use super::departments_sort::sort_rows;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DepartmentsQuery {
    pub q: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

impl DepartmentsQuery {
    pub(super) fn search(&self) -> Option<&str> {
        self.q.as_deref().map(str::trim).filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub kpis: DepartmentsKpiView,
    pub sort_headers: Vec<SortHeaderView>,
    pub search: String,
    pub filters_applied: bool,
    pub clear_url: &'static str,
    pub total: usize,
    pub has_rows: bool,
    pub rows: Vec<DepartmentRowView>,
    pub can_write: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentsKpiView {
    pub departments: usize,
    pub members: i64,
    pub requests_display: String,
    pub tokens_display: String,
    pub cost_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentRowView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub detail_url: String,
    pub member_count: i64,
    pub assignment_count: i64,
    pub requests_display: String,
    pub tokens_display: String,
    pub cost_display: String,
    // Why: the Default department is where deleted departments' members go,
    // so it is the one row that has no delete control.
    pub deletable: bool,
}

pub(super) fn detail_url(id: &str) -> String {
    format!("/admin/departments/{}", urlencoding::encode(id))
}

pub(super) fn kpis(all: &[DepartmentSummary]) -> DepartmentsKpiView {
    DepartmentsKpiView {
        departments: all.len(),
        members: all.iter().map(|d| d.member_count).sum(),
        requests_display: format_token_total(all.iter().map(|d| d.requests).sum()),
        tokens_display: format_token_total(
            all.iter().map(|d| d.input_tokens + d.output_tokens).sum(),
        ),
        cost_display: format_cost(all.iter().map(|d| d.cost_microdollars).sum()),
    }
}

pub(super) fn rows(all: &[DepartmentSummary], query: &DepartmentsQuery) -> Vec<DepartmentRowView> {
    let needle = query.search().map(str::to_lowercase);
    let mut matched: Vec<&DepartmentSummary> = all
        .iter()
        .filter(|d| {
            needle.as_ref().is_none_or(|n| {
                d.name.to_lowercase().contains(n) || d.description.to_lowercase().contains(n)
            })
        })
        .collect();
    sort_rows(&mut matched, query);
    matched
        .into_iter()
        .map(|d| DepartmentRowView {
            id: d.id.clone(),
            name: d.name.clone(),
            description: d.description.clone(),
            detail_url: detail_url(&d.id),
            member_count: d.member_count,
            assignment_count: d.assignment_count,
            requests_display: format_token_total(d.requests),
            tokens_display: format_token_total(d.input_tokens + d.output_tokens),
            cost_display: format_cost(d.cost_microdollars),
            deletable: d.name != DEFAULT_DEPARTMENT,
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentDetailPageData {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub department: Department,
    pub kpis: DepartmentDetailKpiView,
    pub matrix_url: String,
    pub users_url: String,
    pub is_default: bool,
    pub top_tools: Vec<DepartmentTopTool>,
    pub has_tools: bool,
    pub members: Vec<DepartmentMemberRowView>,
    pub member_count: usize,
    pub has_members: bool,
    pub can_write: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentDetailKpiView {
    pub members: usize,
    pub requests_display: String,
    pub tokens_in_display: String,
    pub tokens_out_display: String,
    pub cost_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentMemberRowView {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub detail_url: String,
    pub status: String,
    pub status_tone: &'static str,
    pub requests_display: String,
    pub tokens_display: String,
    pub cost_display: String,
    pub last_active: String,
    pub last_active_title: String,
}

pub(super) fn detail_kpis(members: &[DepartmentMember]) -> DepartmentDetailKpiView {
    DepartmentDetailKpiView {
        members: members.len(),
        requests_display: format_token_total(members.iter().map(|m| m.requests).sum()),
        tokens_in_display: format_token_total(members.iter().map(|m| m.input_tokens).sum()),
        tokens_out_display: format_token_total(members.iter().map(|m| m.output_tokens).sum()),
        cost_display: format_cost(members.iter().map(|m| m.cost_microdollars).sum()),
    }
}

pub(super) fn member_rows(members: &[DepartmentMember]) -> Vec<DepartmentMemberRowView> {
    members
        .iter()
        .map(|m| DepartmentMemberRowView {
            id: m.id.clone(),
            email: m.email.clone(),
            display_name: m.display_name.clone().unwrap_or_default(),
            detail_url: format!("/admin/user?id={}", urlencoding::encode(&m.id)),
            status_tone: if m.status == "active" { "ok" } else { "muted" },
            status: m.status.clone(),
            requests_display: format_token_total(m.requests),
            tokens_display: format_token_total(m.input_tokens + m.output_tokens),
            cost_display: format_cost(m.cost_microdollars),
            last_active: m.last_active.map_or_else(
                || "—".to_owned(),
                |t| t.format("%Y-%m-%d %H:%M").to_string(),
            ),
            last_active_title: m.last_active.map(|t| t.to_rfc3339()).unwrap_or_default(),
        })
        .collect()
}
