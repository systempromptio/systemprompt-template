//! The sortable column set of each tab, and what each column is for.
//!
//! A column is declared once, here, and the same declaration renders the
//! header, builds its sort link and whitelists its key. That is the point:
//! a sort key the reader cannot see in a header cannot be one they clicked,
//! so the list a query parameter is matched against is the list the page
//! actually drew. Every column is an aggregate over a person's rows — the
//! newest, the latest, the sum — because the row it sorts is a person.

use super::view::ColumnSpec;

pub(super) const BRIDGE_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "version",
        label: "Version",
        class: "sp-col-version",
        hint: "Newest bridge build across this person's machines; open the row for each one",
    },
    ColumnSpec {
        key: "started",
        label: "First seen",
        class: "sp-table__cell--date",
        hint: "When this person's first bridge connected",
    },
    ColumnSpec {
        key: "heartbeat",
        label: "Last heartbeat",
        class: "sp-table__cell--date",
        hint: "The most recent heartbeat from any of their machines; silent for a week is stale",
    },
    ColumnSpec {
        key: "forwarded",
        label: "Forwarded",
        class: "sp-table__cell--num",
        hint: "Requests relayed to the gateway, summed over every machine and session",
    },
    ColumnSpec {
        key: "tokens",
        label: "Tokens",
        class: "sp-table__cell--num",
        hint: "Input plus output tokens, summed over every machine and session",
    },
];

pub(super) const PAT_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "created",
        label: "Newest issued",
        class: "sp-table__cell--date",
        hint: "When this person's most recent token was minted",
    },
    ColumnSpec {
        key: "used",
        label: "Last used",
        class: "sp-table__cell--date",
        hint: "The last time any of their tokens was presented",
    },
    ColumnSpec {
        key: "expires",
        label: "Next expiry",
        class: "sp-table__cell--date",
        hint: "The soonest a live token lapses; Never if none of them will",
    },
];

pub(super) const CERT_COLUMNS: &[ColumnSpec] = &[ColumnSpec {
    key: "enrolled",
    label: "Latest enrolled",
    class: "sp-table__cell--date",
    hint: "When this person's most recent certificate was issued",
}];

pub(super) const LINK_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "created",
        label: "Newest issued",
        class: "sp-table__cell--date",
        hint: "When this person's most recent connect code was handed out",
    },
    ColumnSpec {
        key: "expires",
        label: "Latest expiry",
        class: "sp-table__cell--date",
        hint: "Connect codes live ten minutes and are single use",
    },
];
