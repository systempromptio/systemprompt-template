//! Loader for the Salesforce downstream-credential config.
//!
//! Split from the parent module to keep it inside the file-size gate. The
//! config is optional: absent means "not configured", the accessor reports
//! itself unavailable, and the MCP server that depends on it ships disabled.

use std::sync::Arc;

use crate::config_loader::ConfigError;

use super::load_config_section;

// Why: a flat section deserialized into the admin crate's type, absent meaning
// "not configured" rather than an error. It is not required for the server to
// run: the accessor reports itself unavailable and the MCP server that uses it
// ships disabled.
fn load_downstream_config<T: serde::de::DeserializeOwned>(
    filename: &str,
    label: &str,
) -> Result<Option<Arc<T>>, ConfigError> {
    let Some(value) = load_config_section(filename)? else {
        return Ok(None);
    };
    let config: T = serde_yaml::from_value(value).map_err(|e| ConfigError::Parse {
        config_name: filename.to_owned(),
        message: e.to_string(),
    })?;
    tracing::info!(file = %filename, "Loaded {label} config");
    Ok(Some(Arc::new(config)))
}

pub(super) fn load_salesforce_config()
-> Result<Option<Arc<systemprompt_web_admin::SalesforceConfig>>, ConfigError> {
    load_downstream_config("salesforce.yaml", "Salesforce")
}
