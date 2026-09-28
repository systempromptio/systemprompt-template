//! The configuration archive: every plane's database rendered back to its
//! file and zipped, and the reverse — a zip staged for a preview before
//! anything is applied.
//!
//! An export is what the console decided, in the shape the repository
//! keeps it: `services/<plane file>` per plane plus a `MANIFEST.yaml`
//! naming the release, the tree and composition hashes, and per plane the
//! declared and applied hashes at the moment of export. Commit the tree and
//! the code is back in step with the database.
//!
//! An import never writes on upload. The archive is unpacked under strict
//! rules ([`read`]), each plane file is held as text in a staging store
//! ([`staging`]), and the preview page computes every plane's drift from
//! that text through [`super::plane::DeclarationSource::Text`]. Only an
//! explicit apply, per plane and per mode, reaches the database. Entries
//! the instance does not project — skills, marketplaces, providers — are
//! reported for what they are: things to commit or to publish as a bundle.

pub mod manifest;
pub mod read;
pub mod staging;
pub mod write;

pub const MANIFEST_FILE: &str = "MANIFEST.yaml";
pub const TREE_PREFIX: &str = "services/";

// Why: an archive of declarations is kilobytes; the caps exist so a
// malformed or hostile upload is refused early and never allocates its
// declared size. Total is checked as bytes are read, not from headers.
pub const MAX_UPLOAD_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_ENTRY_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 512;
