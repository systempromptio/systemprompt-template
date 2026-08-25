//! Enterprise view-model shaping: seats, budget health, and margin.

use crate::repositories::organizations::crud::OrganizationMember;
use crate::repositories::organizations::detail::{
    OrganizationDepartment, OrganizationEntitlement, OrganizationModelUsage,
};
use crate::repositories::organizations::metrics::OrganizationMetrics;

use super::super::types::{
    BudgetState, EnterpriseDepartmentView, EnterpriseEntitlementView, EnterpriseMemberView,
    EnterpriseModelUsageView, EnterpriseView,
};

pub(super) fn enterprise_view(m: &OrganizationMetrics) -> EnterpriseView {
    let budget_pct = m.budget_used_pct();
    let margin = m.margin_microdollars();

    EnterpriseView {
        slug: m.slug.clone(),
        name: m.name.clone(),
        plan_id: m.plan_id.clone(),
        plan_name: m.plan_name.clone(),
        status: m.status.clone(),
        is_active: m.status == "active",
        is_platform: m.is_platform,
        seats_used: m.seats_used,
        seat_limit: m.seat_limit,
        has_seat_limit: m.seat_limit.is_some_and(|l| l > 0),
        seats_pct: seats_pct(m.seats_used, m.seat_limit),
        departments: m.departments,
        entitlements: m.entitlements,
        requests_30d: m.requests_30d,
        tokens_30d: m.tokens_30d,
        cost_30d_microdollars: m.cost_microdollars_30d,
        cost_mtd_microdollars: m.cost_microdollars_mtd,
        revenue_microdollars: m.revenue_microdollars,
        margin_microdollars: margin,
        margin_positive: margin >= 0,
        has_budget: budget_pct.is_some(),
        budget_pct: budget_pct.unwrap_or_default(),
        budget_state: budget_pct.map_or("none", |p| BudgetState::from_pct(p).as_str()),
    }
}

fn seats_pct(used: i64, limit: Option<i32>) -> i64 {
    let Some(limit) = limit.map(i64::from).filter(|l| *l > 0) else {
        return 0;
    };
    (used.saturating_mul(100) / limit).min(100)
}

pub(super) fn member_view(m: OrganizationMember) -> EnterpriseMemberView {
    EnterpriseMemberView {
        user_id: m.user_id,
        email: m.email,
        display_name: m.display_name,
        org_role: m.org_role,
        department: m.department,
        is_active: m.is_active,
    }
}

pub(super) fn department_view(d: OrganizationDepartment) -> EnterpriseDepartmentView {
    EnterpriseDepartmentView {
        id: d.id,
        name: d.name,
        description: d.description,
        member_count: d.member_count,
    }
}

pub(super) fn entitlement_view(e: OrganizationEntitlement) -> EnterpriseEntitlementView {
    EnterpriseEntitlementView {
        allowed: e.access == "allow",
        entity_type: e.entity_type,
        entity_id: e.entity_id,
        access: e.access,
    }
}

// Why: the share is of cost rather than of requests: a handful of calls to an
// expensive model is the line an operator needs to see, and counting requests
// would bury it under chatter.
pub(super) fn model_views(rows: Vec<OrganizationModelUsage>) -> Vec<EnterpriseModelUsageView> {
    let total: i64 = rows.iter().map(|r| r.cost_microdollars).sum();
    rows.into_iter()
        .map(|r| EnterpriseModelUsageView {
            share_pct: if total > 0 {
                r.cost_microdollars.saturating_mul(100) / total
            } else {
                0
            },
            model: r.model,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost_microdollars,
        })
        .collect()
}
