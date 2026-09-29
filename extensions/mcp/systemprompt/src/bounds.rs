//! Bounds the size of a passthrough CLI result before it goes on the wire.
//!
//! An `infra logs audit` of a long session is a 100-400 KB single line, and a
//! wide `request list` can run to megabytes. Sent whole it exceeds what a host
//! will hand a model as a tool result, and the host then spills it to a file
//! the model cannot read back. The decision here is pure: [`oversize_bytes`]
//! says whether a result fits under [`MAX_RESULT_BYTES`]; the handler stores
//! an oversize result whole as an artifact and sends [`overflow_pointer`] in
//! its place, so nothing is lost and the model gets a link it can act on.
//! [`bound_artifact`] is the last resort when that store fails: trim, and say
//! what was cut.

use systemprompt::identifiers::ArtifactId;
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt_mcp_shared::truncate_on_char_boundary;

// Why: Largest serialized result sent to the model as a tool result. Anything
// over it is stored whole as an artifact and replaced by a pointer.
pub const MAX_RESULT_BYTES: usize = 1024 * 1024;

const NARROW_HINT: &str = "Narrow the query: add --since/--until/--user or a smaller --limit, page \
                           with --before <cursor>, or use the typed tools (user_activity, \
                           conversation_list, request_log, conversation_audit, usage_by_user, \
                           users), which page on their own.";

/// The bounded artifact and whether anything was cut.
#[derive(Debug)]
pub struct Bounded {
    pub artifact: CliArtifact,
    pub truncated: bool,
    // Why: (kept, received) for a table so the summary can say what was cut;
    // other shapes have no row count to report.
    pub rows: Option<(usize, usize)>,
}

/// An oversize result that was stored whole, as the pointer describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredOverflow {
    pub artifact_id: ArtifactId,
    pub bytes: usize,
    pub rows: Option<usize>,
    // Why: past the platform's payload ceiling only the digest is kept, and
    // the pointer must not promise a body the artifact page cannot show.
    pub digest_only: bool,
}

// Why: Serialized size of the artifact as it would go on the wire.
#[must_use]
pub fn wire_bytes(artifact: &CliArtifact) -> usize {
    serde_json::to_vec(artifact).map_or(0, |b| b.len())
}

// Why: The overflow decision: `Some(bytes)` when the result is too big to send.
#[must_use]
pub fn oversize_bytes(artifact: &CliArtifact) -> Option<usize> {
    let bytes = wire_bytes(artifact);
    (bytes > MAX_RESULT_BYTES).then_some(bytes)
}

// Why: The rows an artifact carries, when it is a table.
#[must_use]
pub const fn row_count(artifact: &CliArtifact) -> Option<usize> {
    match artifact {
        CliArtifact::Table { artifact } => Some(artifact.items.len()),
        _ => None,
    }
}

// Why: The short result sent in place of a stored oversize one.
#[must_use]
pub fn overflow_pointer(command: &str, stored: &StoredOverflow) -> CliArtifact {
    let rows = stored
        .rows
        .map_or_else(String::new, |n| format!(" ({n} rows)"));
    let kept = if stored.digest_only {
        "Only its digest was kept because it exceeds the platform's payload ceiling"
    } else {
        "It was stored whole"
    };
    let content = format!(
        "The output of `{command}` is {} bytes{rows}, over the {} byte tool-result bound. {kept} \
         as artifact {} — open /admin/artifacts/{} to read it. {NARROW_HINT}",
        stored.bytes,
        MAX_RESULT_BYTES,
        stored.artifact_id.as_str(),
        stored.artifact_id.as_str(),
    );
    CliArtifact::text(
        TextArtifact::new(content)
            .with_title(format!("Output of `{command}` stored as an artifact")),
    )
}

// Why: a table loses trailing rows (the newest survive); anything else is
// serialized and truncated as text. An artifact that fits is untouched.
#[must_use]
pub fn bound_artifact(artifact: CliArtifact, command: &str) -> Bounded {
    if oversize_bytes(&artifact).is_none() {
        return Bounded {
            artifact,
            truncated: false,
            rows: None,
        };
    }
    match artifact {
        CliArtifact::Table { mut artifact } => {
            let received = artifact.items.len();
            while !artifact.items.is_empty()
                && wire_bytes(&CliArtifact::Table {
                    artifact: artifact.clone(),
                }) > MAX_RESULT_BYTES
            {
                let keep = artifact.items.len() * 3 / 4;
                artifact.items.truncate(keep);
            }
            let kept = artifact.items.len();
            let title = format!(
                "{} — truncated to {kept} of {received} rows. {NARROW_HINT}",
                artifact.title.clone().unwrap_or_else(|| command.to_owned())
            );
            Bounded {
                artifact: CliArtifact::Table {
                    artifact: artifact.with_title(title),
                },
                truncated: true,
                rows: Some((kept, received)),
            }
        },
        CliArtifact::Text { artifact } => Bounded {
            artifact: CliArtifact::text(
                TextArtifact::new(truncate_on_char_boundary(
                    &artifact.content,
                    MAX_RESULT_BYTES,
                ))
                .with_title(format!("Output of `{command}` (truncated). {NARROW_HINT}")),
            ),
            truncated: true,
            rows: None,
        },
        other => {
            let text = match serde_json::to_string(&other) {
                Ok(text) => text,
                Err(error) => format!("unserializable artifact: {error}"),
            };
            let body = truncate_on_char_boundary(&text, MAX_RESULT_BYTES);
            Bounded {
                artifact: CliArtifact::text(
                    TextArtifact::new(body)
                        .with_title(format!("Output of `{command}` (truncated). {NARROW_HINT}")),
                ),
                truncated: true,
                rows: None,
            }
        },
    }
}
