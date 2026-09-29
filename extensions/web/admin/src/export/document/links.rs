//! The download links of one conversation's full record, which the export
//! dialog offers as its "Full transcript" choice.

use serde::Serialize;
use systemprompt::identifiers::ContextId;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DocumentExportView {
    pub json_href: String,
    pub markdown_href: String,
}

impl DocumentExportView {
    pub(crate) fn conversation(context_id: &ContextId) -> Self {
        let id = urlencoding::encode(context_id.as_str());
        Self {
            json_href: format!("/admin/export/transcripts/{id}?format=json"),
            markdown_href: format!("/admin/export/transcripts/{id}?format=markdown"),
        }
    }
}
