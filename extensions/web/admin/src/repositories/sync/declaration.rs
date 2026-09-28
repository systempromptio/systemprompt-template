//! One way to read a plane's declaration text, whichever door it came in.
//!
//! Every plane treats a missing file as an empty declaration — the loader
//! that seeds an instance does, so drift must too — and every plane reads
//! from the composed services root, so a kit that ships the file declares
//! through the same door. Uploaded text skips both: it is the declaration,
//! whole, and the plane parses it as it would the file.

use std::path::PathBuf;

use super::plane::DeclarationSource;
use crate::error::{AdminError, AdminResult};
use crate::repositories::gateway_policies::declared::services_root;

// Why: `None` only for an absent file on disk; uploaded text is always the
// whole declaration.
pub fn read_or_text(
    from: DeclarationSource<'_>,
    source_file: &'static str,
) -> AdminResult<Option<String>> {
    match from {
        DeclarationSource::Text(text) => Ok(Some(text.to_owned())),
        DeclarationSource::Disk => {
            let path: PathBuf = services_root()?.join(source_file);
            match std::fs::read_to_string(&path) {
                Ok(s) => Ok(Some(s)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(AdminError::invalid("a declaration could not be read", e)),
            }
        },
    }
}
