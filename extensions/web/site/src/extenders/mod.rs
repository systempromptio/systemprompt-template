//! Template data extenders that splice deployment-specific values into every
//! page.

mod org_url;
mod release_version;

pub use org_url::OrgUrlExtender;
pub use release_version::ReleaseVersionExtender;
