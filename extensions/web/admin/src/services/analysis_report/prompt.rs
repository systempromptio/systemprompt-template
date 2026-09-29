//! The reporter's one structured-output call: the system prompt, the JSON
//! schema the provider enforces, and the normalisation of what comes back.
//! The model is given only the deterministic digest and asked for ids, never
//! URLs; every evidence link is rebuilt here from those ids.

use crate::repositories::analysis::reports::{
    Assessment, Evidence, Priority, Recommendation, ReportFindings, Severity, Theme, evidence_href,
};
use serde::Deserialize;

pub(super) const SYSTEM_PROMPT: &str = "\
You are an enterprise AI-governance analyst writing for the operator of one AI gateway \
instance. You are given a DIGEST: deterministic figures over the conversations people had \
through the gateway in a window — totals, the skills invoked, the models used, spend by \
person, the best and worst conversations by the judge's 0–100 completion score, cost \
outliers, tool calls the governance chain denied, and the intent and client mix. You never \
see transcripts. Everything you write must be grounded in a figure in the digest; do not \
speculate about causes the digest cannot show, and say when a figure is too small to mean \
anything.

headline: one sentence, at most 25 words, the operator's takeaway.

assessment: ok (nothing needs attention), watch (something is drifting or one figure is \
worth a look), degraded (people are not getting what they ask for, spend is out of \
proportion, or governance is denying real work).

themes: 3 to 8 findings, most important first. Each has kind (adoption | completion | \
cost | errors | governance | skills | models | people | other), severity (ok | warn | err), \
a title of at most 12 words, a detail of at most 60 words citing the figures, and \
evidence: the digest ids it rests on as {kind, id, label} where kind is skill | \
conversation | model | tool | person and id is copied EXACTLY from the digest line \
(the skill key, the conversation id, the model name, the tool name, the person id).

recommendations: 2 to 6 concrete actions the operator can take on this console — which \
skill to fix or retire, which model route to change, whose spend to review, which denied \
tool to allow or block — each with a one-sentence rationale and a priority (high | medium \
| low). Never quote credentials or personal data beyond a display name.";

// JSON: the response schema handed to the provider API — an OpenAPI subset
// (Gemini's responseSchema), so enums and `required` are all it carries.
#[must_use]
pub(crate) fn findings_schema() -> serde_json::Value {
    let evidence = serde_json::json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": ["skill", "conversation", "model", "tool", "person"]},
            "id": {"type": "string"},
            "label": {"type": "string"}
        },
        "required": ["kind", "id", "label"]
    });
    serde_json::json!({
        "type": "object",
        "properties": {
            "headline": {"type": "string"},
            "assessment": {"type": "string", "enum": ["ok", "watch", "degraded"]},
            "themes": {"type": "array", "items": {
                "type": "object",
                "properties": {
                    "kind": {"type": "string"},
                    "severity": {"type": "string", "enum": ["ok", "warn", "err"]},
                    "title": {"type": "string"},
                    "detail": {"type": "string"},
                    "evidence": {"type": "array", "items": evidence}
                },
                "required": ["kind", "severity", "title", "detail", "evidence"]
            }},
            "recommendations": {"type": "array", "items": {
                "type": "object",
                "properties": {
                    "action": {"type": "string"},
                    "rationale": {"type": "string"},
                    "priority": {"type": "string", "enum": ["high", "medium", "low"]}
                },
                "required": ["action", "rationale", "priority"]
            }}
        },
        "required": ["headline", "assessment", "themes", "recommendations"]
    })
}

#[derive(Debug, Deserialize)]
struct RawEvidence {
    kind: String,
    id: String,
    #[serde(default)]
    label: String,
}

#[derive(Debug, Deserialize)]
struct RawTheme {
    kind: String,
    severity: Severity,
    title: String,
    detail: String,
    #[serde(default)]
    evidence: Vec<RawEvidence>,
}

#[derive(Debug, Deserialize)]
struct RawFindings {
    headline: String,
    assessment: Assessment,
    #[serde(default)]
    themes: Vec<RawTheme>,
    #[serde(default)]
    recommendations: Vec<Recommendation>,
}

const MAX_HEADLINE: usize = 160;
const MAX_THEMES: usize = 8;
const MAX_RECOMMENDATIONS: usize = 6;
const MAX_EVIDENCE: usize = 6;
const MAX_TITLE: usize = 120;
const MAX_DETAIL: usize = 600;

fn clip(value: &str, max_chars: usize) -> String {
    value.trim().chars().take(max_chars).collect()
}

// Why: clamps every list and text into the bounds the page expects and drops
// evidence whose id maps to no console page, so a hallucinated id becomes a
// missing chip rather than a dead link.
pub(crate) fn parse_findings(content: &str) -> Result<ReportFindings, serde_json::Error> {
    let raw: RawFindings = serde_json::from_str(content)?;
    let themes = raw
        .themes
        .into_iter()
        .take(MAX_THEMES)
        .map(|t| Theme {
            kind: clip(&t.kind, 40).to_lowercase(),
            severity: t.severity,
            title: clip(&t.title, MAX_TITLE),
            detail: clip(&t.detail, MAX_DETAIL),
            evidence: t
                .evidence
                .into_iter()
                .filter_map(|e| {
                    evidence_href(&e.kind, &e.id).map(|href| Evidence {
                        kind: e.kind,
                        label: if e.label.trim().is_empty() {
                            e.id
                        } else {
                            clip(&e.label, 80)
                        },
                        href,
                    })
                })
                .take(MAX_EVIDENCE)
                .collect(),
        })
        .collect();
    let mut recommendations: Vec<Recommendation> = raw
        .recommendations
        .into_iter()
        .take(MAX_RECOMMENDATIONS)
        .map(|r| Recommendation {
            action: clip(&r.action, 240),
            rationale: clip(&r.rationale, 400),
            priority: r.priority,
        })
        .collect();
    recommendations.sort_by_key(|r| priority_rank(r.priority));
    Ok(ReportFindings {
        headline: clip(&raw.headline, MAX_HEADLINE),
        assessment: raw.assessment,
        themes,
        recommendations,
    })
}

const fn priority_rank(priority: Priority) -> u8 {
    match priority {
        Priority::High => 0,
        Priority::Medium => 1,
        Priority::Low => 2,
    }
}
