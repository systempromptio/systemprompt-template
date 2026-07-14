//! Department listing + detail view models for the management pages.
//!
//! Holds the serde page-data shapes and the member-rollup arithmetic that the
//! `management-departments` / `management-department-detail` templates consume.

use serde::Serialize;

use crate::types::departments::{
    Department, DepartmentMember, DepartmentSummary, DepartmentTopTool,
};

pub(super) fn url_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            },
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub departments: Vec<DepartmentSummary>,
}

#[derive(Debug, Serialize)]
pub(super) struct DepartmentDetailPageData {
    pub page: &'static str,
    pub title: String,
    pub department: Department,
    pub members: Vec<DepartmentMember>,
    pub member_count: i64,
    pub assignments_url: String,
    pub top_tools: Vec<DepartmentTopTool>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_requests: i64,
    pub total_cost_microdollars: i64,
}

#[derive(Debug, Default)]
pub(super) struct MemberTotals {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
}

pub(super) fn sum_member_totals(members: &[DepartmentMember]) -> MemberTotals {
    let mut t = MemberTotals::default();
    for m in members {
        t.input_tokens += m.input_tokens;
        t.output_tokens += m.output_tokens;
        t.requests += m.requests;
        t.cost_microdollars += m.cost_microdollars;
    }
    t
}
