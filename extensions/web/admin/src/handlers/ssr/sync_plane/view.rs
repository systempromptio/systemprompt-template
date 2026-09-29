//! View types for the sync-plane component, shared by every page that
//! renders one: an owner page's Sync tab, the import preview, and the
//! configuration page's state column.

use serde::Serialize;

use crate::repositories::sync::plane::{ActionCounts, DriftKpi, DriftRow};

// Why: A hash as the page prints it: the first twelve characters, with the full
// value in a title attribute.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct HashView {
    pub short: String,
    pub full: String,
}

impl HashView {
    pub(crate) fn of(full: &str) -> Option<Self> {
        (!full.is_empty()).then(|| Self {
            short: full.chars().take(12).collect(),
            full: full.to_owned(),
        })
    }
}

// Why: "last applied ab12… by Ed on 16 Sep (overwrite)"; `declared_changed`
// says the declaration's hash differs from the one last applied.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AppliedView {
    pub hash: Option<HashView>,
    pub mode: String,
    pub at: String,
    pub by: String,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub declared_changed: bool,
}

// Why: the one line under the tiles that says what to press. `card` names
// the direction card it points at (`overwrite` or `export`) so the template
// can highlight it; a clean plane has none.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncRecommendation {
    pub text: String,
    pub card: &'static str,
}

impl SyncRecommendation {
    pub(crate) fn for_actions(actions: &ActionCounts, is_clean: bool) -> Option<Self> {
        if is_clean {
            return None;
        }
        let code = actions.code_side();
        let console = actions.kept + actions.delete_console;
        // Why: an entity the file declares and the database has never heard of
        // is the one drift boot cannot settle by itself. Boot seeds a plane
        // only when its projection is empty, and for access control "empty"
        // counts band rules, not entities — a database with 63 entities and
        // 159 rules can never seed, so entities added by a later release stay
        // absent and the console is the only route in. Apply all new adds them
        // without touching anything written here, so it is named first and
        // separately from the general code-side wording, which reads as an
        // Overwrite recommendation.
        if actions.insert_entities > 0 {
            let deletes = if actions.delete > 0 {
                format!(
                    " Replace database with code would also delete {} row(s), so use it only if that is what you want.",
                    actions.delete
                )
            } else {
                String::new()
            };
            return Some(Self {
                text: format!(
                    "{} entity(s) declared in code are absent from this database; Apply all new adds them, with {} row(s), and changes nothing written here.{deletes}",
                    actions.insert_entities, actions.insert
                ),
                card: "insert",
            });
        }
        let moves = format!(
            "+{} · ~{} · −{}",
            actions.insert + actions.insert_entities,
            actions.update,
            actions.delete
        );
        let text = match (code, console) {
            (true, 0) => {
                format!("Replace database with code brings this database in step ({moves}).")
            },
            (false, n) => format!(
                "{n} row(s) exist only in this database. Export it and commit the file to keep them; nothing here is deleted by code."
            ),
            (true, n) => format!(
                "Replace database with code settles the code side ({moves}); {n} row(s) written here need Export to reach the file first if they are to be kept."
            ),
        };
        Some(Self {
            text,
            card: if code && actions.delete_console == 0 {
                "overwrite"
            } else {
                "export"
            },
        })
    }
}

// Why: `apply_url` is where the buttons post — the plane's own apply, or a
// staged import's — and `show_export` is off on a preview, where the
// database is not the thing being read.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlaneCardView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub projection: &'static str,
    pub owner_url: &'static str,
    pub runtime_note: Option<&'static str>,
    pub declared_hash: Option<HashView>,
    pub declared_count: usize,
    pub in_db: usize,
    pub kpis: Vec<DriftKpi>,
    pub actions: ActionCounts,
    pub recommendation: Option<SyncRecommendation>,
    pub rows: Vec<DriftRow>,
    pub row_count: usize,
    pub is_clean: bool,
    pub unreadable: Option<String>,
    pub applied: Option<AppliedView>,
    pub export_url: String,
    pub apply_url: String,
    pub show_export: bool,
    pub entity_filter: String,
    pub clear_filter_url: String,
    pub source_label: &'static str,
}
