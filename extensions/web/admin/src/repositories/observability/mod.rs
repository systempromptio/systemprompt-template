//! What the profile declares under `observability.otlp`.
//!
//! Core streams telemetry to the declared collector itself; an install reads
//! only the target, without its headers or URL credentials, from the profile
//! view. `None` is a profile that declares no collector, a state the page
//! names, not an error.

use crate::env::AdminEnv;

pub mod view;

use systemprompt::manifest::profile::OtlpTargetView;


#[must_use]
pub fn find_declared_export(env: &AdminEnv) -> Option<OtlpTargetView> {
    env.profile().otlp.clone()
}
