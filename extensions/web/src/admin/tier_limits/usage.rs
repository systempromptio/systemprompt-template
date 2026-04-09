use serde::Serialize;

use super::{TierLimits, UsageWarning};
use crate::admin::numeric;

#[derive(Debug, Clone, Default, Serialize, Copy)]
pub struct UsageSnapshot {
    pub events_today: i64,
    pub content_bytes_today: i64,
    pub sessions_today: i64,
    pub skills_count: i64,
    pub agents_count: i64,
    pub plugins_count: i64,
    pub mcp_servers_count: i64,
    pub hooks_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub limits: TierLimits,
    pub usage: UsageSnapshot,
    pub plan_name: String,
    pub warnings: Vec<UsageWarning>,
}

impl UsageSummary {
    #[must_use]
    pub fn build(limits: TierLimits, usage: UsageSnapshot, plan_name: String) -> Self {
        let mut warnings = Vec::new();

        let checks = [
            (
                "events",
                "Daily events",
                usage.events_today,
                limits.ingestion.events,
            ),
            (
                "content_bytes",
                "Daily data",
                usage.content_bytes_today,
                limits.ingestion.content_bytes,
            ),
            (
                "sessions",
                "Daily sessions",
                usage.sessions_today,
                limits.ingestion.sessions,
            ),
            (
                "skills",
                "Skills",
                usage.skills_count,
                limits.entities.skills,
            ),
            (
                "agents",
                "Agents",
                usage.agents_count,
                limits.entities.agents,
            ),
            (
                "plugins",
                "Plugins",
                usage.plugins_count,
                limits.entities.plugins,
            ),
            (
                "mcp_servers",
                "MCP Servers",
                usage.mcp_servers_count,
                limits.entities.mcp_servers,
            ),
            ("hooks", "Hooks", usage.hooks_count, limits.entities.hooks),
        ];

        for (category, label, current, limit) in checks {
            if limit > 0 {
                let pct = numeric::to_f64(current) / numeric::to_f64(limit);
                if pct >= 1.0 {
                    warnings.push(UsageWarning {
                        category: category.to_string(),
                        message: format!(
                            "{label} limit reached ({current}/{limit}). Upgrade to continue."
                        ),
                        usage_pct: pct,
                    });
                } else if pct >= 0.8 {
                    warnings.push(UsageWarning {
                        category: category.to_string(),
                        message: format!("{label} approaching limit ({current}/{limit})."),
                        usage_pct: pct,
                    });
                }
            }
        }

        Self {
            limits,
            usage,
            plan_name,
            warnings,
        }
    }
}
