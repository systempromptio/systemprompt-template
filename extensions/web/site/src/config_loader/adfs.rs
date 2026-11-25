//! Loader for the ADFS SSO config (`services/web/config/adfs*.yaml`).
//!
//! Split from the parent module to keep it inside the file-size gate. The two
//! SAML URLs are derived here from the profile rather than authored in YAML,
//! so one committed config is correct on every deployment host.

use std::path::Path;
use std::sync::Arc;

use super::{ConfigError, load_config_section};

// Why: the relying-party identifier and the ACS are the instance's own public
// URLs, and the profile already names that host. Deriving both from
// `api_external_url` keeps one committed `adfs.yaml` correct on every
// deployment and makes it impossible for them to disagree with each other; a
// tracked literal was right on exactly one host and produced AD FS `MSIS7007`
// on all the others. An explicit value still wins, for a trust registered
// against a URL this instance does not serve itself under.
fn derive_saml_urls(config: &mut systemprompt_web_admin::AdfsConfig, api_external_url: &str) {
    if !config.entity_id.is_empty() && !config.acs_url.is_empty() {
        return;
    }
    let base = api_external_url.trim_end_matches('/');
    if base.is_empty() {
        return;
    }
    if config.entity_id.is_empty() {
        config.entity_id = format!("{base}{}", systemprompt_web_admin::ADFS_METADATA_PATH);
    }
    if config.acs_url.is_empty() {
        config.acs_url = format!("{base}{}", systemprompt_web_admin::ADFS_ACS_PATH);
    }
    tracing::info!(
        entity_id = %config.entity_id,
        acs_url = %config.acs_url,
        "Derived ADFS SAML URLs from the profile"
    );
}

pub(super) fn load_adfs_config(
    services: &Path,
    api_external_url: &str,
) -> Result<Option<Arc<systemprompt_web_admin::AdfsConfig>>, ConfigError> {
    let filename = "adfs.yaml".to_owned();
    let Some(value) = load_config_section(services, &filename)? else {
        return Ok(None);
    };

    let mut config: systemprompt_web_admin::AdfsConfig =
        serde_yaml::from_value(value).map_err(|e| ConfigError::Parse {
            config_name: filename.clone(),
            message: e.to_string(),
        })?;

    derive_saml_urls(&mut config, api_external_url);

    // Why: the IdP signing certificate is pinned in the repository as the
    // federation metadata file; a missing file is a config error, not a
    // silent "SSO unavailable".
    if config.enabled {
        let metadata_path = services.join("web/config").join(&config.idp_metadata_path);
        config.idp_metadata_xml =
            std::fs::read_to_string(&metadata_path).map_err(|e| ConfigError::Parse {
                config_name: filename.clone(),
                message: format!(
                    "idp_metadata_path {} unreadable: {e}",
                    metadata_path.display()
                ),
            })?;
    }

    tracing::info!(
        enabled = config.enabled,
        file = %filename,
        "Loaded ADFS SSO config"
    );

    Ok(Some(Arc::new(config)))
}
