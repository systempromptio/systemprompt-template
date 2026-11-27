//! Wire values built from the JSON the instance writes.

use serde_json::json;
use sp_protocol::wire::brain::{BrainFact, BrainProposal, FactPage, FactStatus};

pub(crate) const FACT_A: &str = "m_6f1c2a5e-3b7d-4c1e-9a8f-0d2b4e6f8a10";
pub(crate) const FACT_B: &str = "m_7a2d3b6f-4c8e-4d2f-8b9a-1e3c5f7a9b21";
pub(crate) const FACT_C: &str = "m_8b3e4c7a-5d9f-4e3a-9cab-2f4d6a8bac32";
pub(crate) const PROPOSAL_OPEN: &str = "0b6f2d4e-1a3c-4e5f-8a7b-9c0d1e2f3a4b";
pub(crate) const PROPOSAL_DONE: &str = "1c7a3e5f-2b4d-4f6a-9b8c-ad1e2f3a4b5c";

pub(crate) fn fact(id: &str, status: FactStatus, pinned: bool) -> BrainFact {
    serde_json::from_value(json!({
        "id": id,
        "kind": "decision",
        "subject": { "kind": "client", "id": "acme" },
        "scope": { "level": "org" },
        "statement": "Acme invoices quarterly",
        "fields": {},
        "source": { "type": "entered_by", "user_id": "user-1" },
        "topics": [],
        "status": status.as_str(),
        "confirmed_by": null,
        "confirmed_at": null,
        "importance_base": 0.5,
        "observed_at": "2026-01-01T00:00:00Z",
        "valid_from": null,
        "valid_to": null,
        "review_after": null,
        "retention_until": null,
        "legal_hold": false,
        "identity_hash": "identity",
        "statement_hash": "statement",
        "revision": 1,
        "pinned": pinned,
        "owner_user_id": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z"
    }))
    .expect("the fixture decodes as a fact")
}

pub(crate) fn page(ids: &[&str], next: Option<&str>) -> FactPage {
    FactPage {
        facts: ids
            .iter()
            .map(|id| fact(id, FactStatus::Confirmed, false))
            .collect(),
        next: next.map(str::to_owned),
    }
}

pub(crate) fn proposal(id: &str, status: &str) -> BrainProposal {
    serde_json::from_value(json!({
        "id": id,
        "analysis": "duplicates",
        "kind": "fact_actions",
        "status": status,
        "summary": "Merge two facts",
        "body": { "actions": [] },
        "created_at": "2026-01-01T00:00:00Z",
        "decided_by": null,
        "decided_at": null
    }))
    .expect("the fixture decodes as a proposal")
}
