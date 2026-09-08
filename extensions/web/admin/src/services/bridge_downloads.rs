//! Optional operator-provided desktop client artifacts.

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BridgeDownloads {
    download_base: Option<String>,
}

// Why: the template does not ship desktop binaries. An unset or invalid
// artifact location must not produce links to files the server cannot serve.
pub(crate) fn download_base(gateway: &str) -> Option<String> {
    let path = crate::handlers::shared::get_services_path()
        .ok()?
        .join("web/config/bridge.yaml");
    let text = std::fs::read_to_string(path).ok()?;
    let config: BridgeDownloads = serde_yaml::from_str(&text)
        .inspect_err(|error| tracing::warn!(%error, "Invalid bridge download configuration"))
        .ok()?;
    let configured = config.download_base?.trim().to_owned();
    if configured.is_empty() {
        return None;
    }
    let url = if configured.starts_with('/') && !configured.starts_with("//") {
        url::Url::parse(gateway).ok()?.join(&configured).ok()?
    } else {
        url::Url::parse(&configured).ok()?
    };
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        tracing::warn!("Bridge download_base must be an HTTP(S) artifact directory");
        return None;
    }
    Some(url.as_str().trim_end_matches('/').to_owned())
}

pub(crate) fn install_command(gateway: &str, code: Option<&str>, host: &str) -> Option<String> {
    let base = download_base(gateway)?;
    let script = shell_arg(&format!("{base}/install.sh"));
    let base = shell_arg(&base);
    let gateway = shell_arg(gateway);
    let code = code
        .map(|code| format!(" --code {}", shell_arg(code)))
        .unwrap_or_default();
    let host = shell_arg(host);
    Some(format!(
        "curl -fsSL {script} | sh -s -- --download-base {base} --gateway {gateway}{code} --host {host}"
    ))
}

fn shell_arg(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
