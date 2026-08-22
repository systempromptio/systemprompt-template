//! Filter-bar assembly for the dashboard: org/department selects, the
//! day/week bucket toggle, and the hidden fields the GET form must carry.
//! Split from `mod.rs` at the 300-line ceiling.

use sqlx::PgPool;

use crate::repositories::analytics::site::SiteScope;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::repositories::departments::list_department_names;
use crate::repositories::organizations::crud;
use crate::types::UserContext;
use crate::util::org_scope::OrgScope;

use super::context::{DashboardTab, FiltersView, HiddenFieldView, SelectOptionView};
use super::{AnalyticsDashboardQuery, urls};

pub(super) async fn build_filters(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &AnalyticsDashboardQuery,
    scope: &SiteScope,
    bucket: SeriesBucket,
) -> FiltersView {
    let org_options = if user_ctx.is_platform_admin {
        let orgs = crud::list_organizations(pool).await.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_organizations failed");
            Vec::new()
        });
        let mut options = vec![SelectOptionView {
            value: String::new(),
            label: "All organizations".to_owned(),
            selected: scope.org_slug == OrgScope::AllOrganizations,
        }];
        options.extend(orgs.into_iter().map(|o| SelectOptionView {
            selected: scope.org_slug.as_slug() == Some(o.slug.as_str()),
            value: o.slug,
            label: o.name,
        }));
        options
    } else {
        Vec::new()
    };

    // Why: the dropdown follows the resolved scope rather than listing every
    // department. The rows behind the filter are already scoped, but the
    // option labels themselves are a customer's internal structure.
    let departments = list_department_names(pool, &scope.org_slug)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_department_names failed");
            Vec::new()
        });
    let mut department_options = vec![SelectOptionView {
        value: String::new(),
        label: "All departments".to_owned(),
        selected: scope.department.is_none(),
    }];
    department_options.extend(departments.into_iter().map(|d| SelectOptionView {
        selected: scope.department.as_deref() == Some(d.as_str()),
        value: d.clone(),
        label: d,
    }));

    // Why: the filter form is a plain GET, so everything not expressed by its
    // selects must ride along as hidden fields or submitting it would reset
    // the tab and window.
    let mut hidden = vec![HiddenFieldView {
        name: "tab",
        value: DashboardTab::from_query(query.tab.as_deref())
            .as_str()
            .to_owned(),
    }];
    if let Some(preset) = query.preset.as_deref().filter(|s| !s.is_empty()) {
        hidden.push(HiddenFieldView {
            name: "preset",
            value: preset.to_owned(),
        });
    }
    if let Some(from) = query.from.as_deref().filter(|s| !s.is_empty()) {
        hidden.push(HiddenFieldView {
            name: "from",
            value: from.to_owned(),
        });
    }
    if let Some(to) = query.to.as_deref().filter(|s| !s.is_empty()) {
        hidden.push(HiddenFieldView {
            name: "to",
            value: to.to_owned(),
        });
    }
    if let Some(b) = query.bucket.as_deref().filter(|s| !s.is_empty()) {
        hidden.push(HiddenFieldView {
            name: "bucket",
            value: b.to_owned(),
        });
    }

    FiltersView {
        show_org_select: user_ctx.is_platform_admin,
        org_options,
        department_options,
        bucket_links: urls::bucket_links(query, bucket == SeriesBucket::Week),
        hidden,
    }
}
