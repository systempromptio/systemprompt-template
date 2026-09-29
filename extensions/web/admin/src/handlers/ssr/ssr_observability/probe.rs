//! "Test connection": one empty OTLP/HTTP envelope to the collector.
//!
//! An `ExportTraceServiceRequest` with no resource spans encodes to zero
//! protobuf bytes, so the probe is the exporter's exact request shape — same
//! URL, same headers, same content type — carrying nothing. A collector that
//! accepts it (2xx) will accept a real batch; one that refuses tells the
//! operator why before the job spends a tick finding out. Nothing here
//! exports: the body is empty and no watermark moves.

use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use systemprompt::models::profile::{OtlpExportConfig, OtlpProtocol, OtlpSignal};

const CONTENT_TYPE_PROTOBUF: &str = "application/x-protobuf";
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const BODY_PREVIEW: usize = 256;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProbeError {
    #[error("only OTLP/HTTP can be probed from the console; this exporter is {0}")]
    Protocol(&'static str),
    #[error("invalid header {name}: {reason}")]
    Header { name: String, reason: String },
    #[error("collector answered {status}: {body}")]
    Status { status: StatusCode, body: String },
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub(crate) struct ProbeOutcome {
    pub url: String,
    pub status: StatusCode,
    pub elapsed_ms: u128,
}

fn build_headers(config: &OtlpExportConfig) -> Result<HeaderMap, ProbeError> {
    let mut headers = HeaderMap::with_capacity(config.headers.len() + 1);
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static(CONTENT_TYPE_PROTOBUF),
    );
    for (name, value) in &config.headers {
        let key = HeaderName::from_bytes(name.as_bytes()).map_err(|e| ProbeError::Header {
            name: name.clone(),
            reason: e.to_string(),
        })?;
        let value = HeaderValue::from_str(value).map_err(|e| ProbeError::Header {
            name: name.clone(),
            reason: e.to_string(),
        })?;
        headers.insert(key, value);
    }
    Ok(headers)
}

pub(crate) async fn probe(config: &OtlpExportConfig) -> Result<ProbeOutcome, ProbeError> {
    if config.protocol != OtlpProtocol::Http {
        return Err(ProbeError::Protocol(config.protocol.label()));
    }
    let url = config.signal_url(OtlpSignal::Traces);
    let headers = build_headers(config)?;
    let started = std::time::Instant::now();
    let response = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()?
        .post(&url)
        .headers(headers)
        .body(Vec::new())
        .send()
        .await?;
    let status = response.status();
    if status.is_success() {
        return Ok(ProbeOutcome {
            url,
            status,
            elapsed_ms: started.elapsed().as_millis(),
        });
    }
    let body = match response.text().await {
        Ok(text) => text.chars().take(BODY_PREVIEW).collect(),
        Err(error) => format!("<body unreadable: {error}>"),
    };
    Err(ProbeError::Status { status, body })
}
