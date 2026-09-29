//! The digest an AI report is written from — `digest.sql` shaped into typed
//! rows — and its rendering as the compact text the model is handed. The
//! text is deterministic for a given digest so a regenerated report answers
//! the same evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;

/// What narrows the record for one report.
#[derive(Debug, Clone, Default)]
pub struct DigestScope {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub subject_ids: Option<Vec<String>>,
    // Why: raw text, not typed ids — a filter query's values are bound
    // straight into the digest SQL and echoed into the stored inputs.
    pub marketplace_key: Option<String>,
    pub skill: Option<String>,
    pub user_key: Option<String>,
    pub model: Option<String>,
    pub client_kind: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DigestTotals {
    pub conversations: i64,
    pub people: i64,
    pub turns: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub denied: i64,
    pub safety_findings: i64,
    pub artifacts: i64,
    pub tool_calls: i64,
    pub judged: i64,
    pub completion_avg: Option<f64>,
    pub achieved: i64,
    pub partial: i64,
    pub abandoned: i64,
    pub unclear: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSkill {
    pub skill: String,
    pub invocations: i64,
    pub conversations: i64,
    pub cost_microdollars: i64,
    pub completion_avg: Option<f64>,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestModel {
    pub model: String,
    pub conversations: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub completion_avg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestPerson {
    #[serde(rename = "user_id")]
    pub user_key: String,
    pub display_name: Option<String>,
    pub conversations: i64,
    pub cost_microdollars: i64,
    pub completion_avg: Option<f64>,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestJudged {
    #[serde(rename = "context_id")]
    pub context_key: String,
    pub title: String,
    pub judge_title: Option<String>,
    pub completion: Option<i16>,
    pub outcome: Option<String>,
    pub category: Option<String>,
    pub cost_microdollars: i64,
    pub turns: i64,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestOutlier {
    #[serde(rename = "context_id")]
    pub context_key: String,
    pub title: String,
    pub cost_microdollars: i64,
    pub turns: i64,
    pub tokens: i64,
    pub completion: Option<i16>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestDenial {
    pub tool_name: String,
    pub denied: i64,
    pub people: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestIntent {
    pub category: String,
    pub conversations: i64,
    pub completion_avg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestClient {
    pub client_kind: String,
    pub conversations: i64,
    pub cost_microdollars: i64,
}

/// Everything the model sees, and everything the report page shows as its
/// evidence base.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportDigest {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub totals: DigestTotals,
    #[serde(default)]
    pub skills: Vec<DigestSkill>,
    #[serde(default)]
    pub models: Vec<DigestModel>,
    #[serde(default)]
    pub people: Vec<DigestPerson>,
    #[serde(default)]
    pub worst: Vec<DigestJudged>,
    #[serde(default)]
    pub best: Vec<DigestJudged>,
    #[serde(default)]
    pub outliers: Vec<DigestOutlier>,
    #[serde(default)]
    pub denials: Vec<DigestDenial>,
    #[serde(default)]
    pub intents: Vec<DigestIntent>,
    #[serde(default)]
    pub clients: Vec<DigestClient>,
}

pub async fn get_report_digest(
    pool: &PgPool,
    scope: &DigestScope,
) -> Result<ReportDigest, sqlx::Error> {
    let row = sqlx::query_file!(
        "src/repositories/analysis/reports/digest.sql",
        scope.window_start,
        scope.window_end,
        scope.subject_ids.as_deref(),
        scope.marketplace_key,
        scope.skill,
        scope.user_key,
        scope.model,
        scope.client_kind,
        scope.category,
        scope.outcome,
    )
    .fetch_one(pool)
    .await?;
    let Json(digest) = row.payload;
    Ok(digest)
}

use systemprompt_web_shared::format::format_cost as usd;

fn score(avg: Option<f64>) -> String {
    avg.map_or_else(|| "n/a".to_owned(), |v| format!("{v:.0}"))
}

// Why: one line per entity with its id first, so the model can cite an id
// verbatim and the report builder can turn it back into a console link.
#[must_use]
pub fn render_digest_text(d: &ReportDigest) -> String {
    let t = &d.totals;
    let mut out = String::new();
    out.push_str(&format!(
        "WINDOW {} → {}\nTOTALS conversations={} people={} turns={} tokens={} cost={} failed_requests={} denied_tool_calls={} safety_findings={} tool_calls={} artifacts={} judged={} mean_completion={} achieved={} partial={} abandoned={} unclear={}\n",
        d.window_start.format("%Y-%m-%d"), d.window_end.format("%Y-%m-%d"),
        t.conversations, t.people, t.turns, t.tokens, usd(t.cost_microdollars), t.errors,
        t.denied, t.safety_findings, t.tool_calls, t.artifacts, t.judged,
        score(t.completion_avg), t.achieved, t.partial, t.abandoned, t.unclear
    ));
    out.push_str("\nSKILLS (id | invocations | conversations | cost | mean_completion | errors)\n");
    for s in &d.skills {
        out.push_str(&format!(
            "skill {} | {} | {} | {} | {} | {}\n",
            s.skill,
            s.invocations,
            s.conversations,
            usd(s.cost_microdollars),
            score(s.completion_avg),
            s.errors
        ));
    }
    out.push_str("\nMODELS (id | conversations | requests | cost | failed | mean_completion)\n");
    for m in &d.models {
        out.push_str(&format!(
            "model {} | {} | {} | {} | {} | {}\n",
            m.model,
            m.conversations,
            m.requests,
            usd(m.cost_microdollars),
            m.errors,
            score(m.completion_avg)
        ));
    }
    out.push_str("\nPEOPLE (id | name | conversations | cost | mean_completion | errors)\n");
    for p in &d.people {
        out.push_str(&format!(
            "person {} | {} | {} | {} | {} | {}\n",
            p.user_key,
            p.display_name.as_deref().unwrap_or("-"),
            p.conversations,
            usd(p.cost_microdollars),
            score(p.completion_avg),
            p.errors
        ));
    }
    push_judged(&mut out, "WORST JUDGED", &d.worst);
    push_judged(&mut out, "BEST JUDGED", &d.best);
    push_mix(&mut out, d);
    out
}

fn push_judged(out: &mut String, heading: &str, rows: &[DigestJudged]) {
    out.push_str(&format!(
        "\n{heading} (id | title | completion | outcome | intent | cost | turns | summary)\n"
    ));
    for j in rows {
        out.push_str(&format!(
            "conversation {} | {} | {} | {} | {} | {} | {} | {}\n",
            j.context_key,
            j.judge_title.as_deref().unwrap_or(&j.title),
            j.completion
                .map_or_else(|| "n/a".to_owned(), |c| c.to_string()),
            j.outcome.as_deref().unwrap_or("-"),
            j.category.as_deref().unwrap_or("-"),
            usd(j.cost_microdollars),
            j.turns,
            j.summary.as_deref().unwrap_or("")
        ));
    }
}

// Why: the second half of the digest text — outliers, denials and the two
// mixes — kept beside the first so the format reads as one document.
fn push_mix(out: &mut String, d: &ReportDigest) {
    out.push_str("\nCOST OUTLIERS (id | title | cost | turns | tokens | completion | model)\n");
    for o in &d.outliers {
        out.push_str(&format!(
            "conversation {} | {} | {} | {} | {} | {} | {}\n",
            o.context_key,
            o.title,
            usd(o.cost_microdollars),
            o.turns,
            o.tokens,
            o.completion
                .map_or_else(|| "n/a".to_owned(), |c| c.to_string()),
            o.model.as_deref().unwrap_or("-")
        ));
    }
    out.push_str("\nDENIED TOOL CALLS (tool | denied | people)\n");
    for x in &d.denials {
        out.push_str(&format!(
            "tool {} | {} | {}\n",
            x.tool_name, x.denied, x.people
        ));
    }
    out.push_str("\nINTENT MIX (category | conversations | mean_completion)\n");
    for i in &d.intents {
        out.push_str(&format!(
            "{} | {} | {}\n",
            i.category,
            i.conversations,
            score(i.completion_avg)
        ));
    }
    out.push_str("\nCLIENT MIX (client | conversations | cost)\n");
    for c in &d.clients {
        out.push_str(&format!(
            "{} | {} | {}\n",
            c.client_kind,
            c.conversations,
            usd(c.cost_microdollars)
        ));
    }
}
