//! Reads `services/access-control/rules.yaml` from disk.
//!
//! Reading is separate from projecting so the drift engine, the sync page and
//! the boot path all parse the file exactly once and in exactly one way. A
//! missing file is `None` — an instance may legitimately ship no rules — while
//! a malformed or structurally invalid file is an error the caller must not
//! paper over: it is the operator's declaration and half of it is no
//! declaration at all.

use std::path::Path;

use systemprompt_web_shared::error::MarketplaceError;

use super::rules_yaml_types::RulesDoc;

pub const RULES_FILE: &str = "access-control/rules.yaml";

pub async fn read_rules_doc(services_path: &Path) -> Result<Option<RulesDoc>, MarketplaceError> {
    let path = services_path.join(RULES_FILE);
    let text = match tokio::fs::read_to_string(&path).await {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    parse_rules_doc(&text).map(Some)
}

pub fn parse_rules_doc(text: &str) -> Result<RulesDoc, MarketplaceError> {
    if text.trim().is_empty() {
        return Ok(RulesDoc::default());
    }
    let doc: RulesDoc =
        serde_yaml::from_str(text).map_err(|e| MarketplaceError::config_file(RULES_FILE, e))?;
    doc.validate()
        .map_err(|e| MarketplaceError::config_file(RULES_FILE, e))?;
    Ok(doc)
}
