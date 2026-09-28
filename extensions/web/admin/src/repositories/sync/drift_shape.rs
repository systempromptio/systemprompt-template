//! The drift shape every sync plane renders.
//!
//! The tiles above the table, one row per line of the declared-versus-database
//! diff, the numbers the action buttons quote, and a declaration rendered back
//! out as its file.

use serde::Serialize;

/// One tile above the drift table.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DriftKpi {
    pub label: &'static str,
    pub value: usize,
    pub note: &'static str,
    pub tone: &'static str,
}

/// One line of a plane's diff table. `kind` is a stable token the page and
/// the tests key on; the labels are for people. `resolve` names the one
/// action that settles the line — see [`resolution`].
#[derive(Debug, Clone, Serialize)]
pub struct DriftRow {
    pub kind: &'static str,
    pub kind_label: &'static str,
    pub kind_tone: &'static str,
    pub entity_type: String,
    pub entity_type_label: String,
    pub entity_id: String,
    pub band: String,
    pub band_label: &'static str,
    pub subject: String,
    pub in_code: String,
    pub in_db: String,
    pub origin: &'static str,
    pub origin_tone: &'static str,
    pub governed: bool,
    pub insert_applies: bool,
    pub overwrite_applies: bool,
    pub overwrite_effect: &'static str,
    pub resolve: String,
    pub resolve_tone: &'static str,
}

// Why: the "Resolves with" badge for a line, from what the two writing
// directions would do to it. One badge per line, so the reader learns
// which button to press without weighing two columns against each other.
#[must_use]
pub fn resolution(
    insert_applies: bool,
    overwrite_applies: bool,
    overwrite_effect: &str,
) -> (String, &'static str) {
    match (insert_applies, overwrite_applies, overwrite_effect) {
        (true, _, _) => ("Insert or Overwrite adds".to_owned(), "ok"),
        (false, true, "deleted") => ("Overwrite deletes".to_owned(), "err"),
        (false, true, "updated") => ("Overwrite corrects".to_owned(), "warn"),
        (false, true, "reordered") => ("Overwrite reorders".to_owned(), "warn"),
        (false, true, effect) => (format!("Overwrite: {effect}"), "warn"),
        (false, false, effect) => (effect.to_owned(), "muted"),
    }
}

/// The numbers the action buttons and their confirm dialog quote.
///
/// What *Insert only* adds (rows, entity defaults), what *Overwrite* corrects
/// and deletes, how many of the deletions the console wrote, and how many
/// database-only rows *Overwrite* keeps because the console or a bundle wrote
/// them on an entity the file does not name — the rows only *Export* carries
/// back to code.
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct ActionCounts {
    pub insert: usize,
    pub insert_entities: usize,
    pub update: usize,
    pub delete: usize,
    pub delete_console: usize,
    pub kept: usize,
}

impl ActionCounts {
    // Why: whether *Overwrite from code* would move anything.
    #[must_use]
    pub const fn code_side(&self) -> bool {
        self.insert + self.insert_entities + self.update + self.delete > 0
    }
}

/// A plane's declared-versus-database picture, ready to render. `unreadable`
/// carries the message when the declaration could not be read; it is the
/// card's headline, not an error.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PlaneDrift {
    pub declared_hash: String,
    pub declared_count: usize,
    pub in_db: usize,
    pub kpis: Vec<DriftKpi>,
    pub actions: ActionCounts,
    pub rows: Vec<DriftRow>,
    pub is_clean: bool,
    pub unreadable: Option<String>,
}

/// A declaration rendered back out of the database as its file.
#[derive(Debug, Clone, Serialize)]
pub struct Export {
    pub filename: &'static str,
    pub content_type: &'static str,
    pub body: String,
    pub row_count: usize,
}
