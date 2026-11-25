//! The template's composition, admitted the way the binary admits it under a
//! profile that disables nothing.
//!
//! Suites that read what a member contributed (its jobs, assets, cross-member
//! tables, site auth) read it from the admitted member, so they see what
//! admission kept rather than what a hand-built contribution sink holds.

use std::sync::Arc;

use systemprompt::Declared;
use systemprompt::extension::Admitted;
use systemprompt::extension::admitted::AdmittedMember;
use systemprompt::traits::Job;

#[must_use]
pub fn admitted() -> Admitted {
    systemprompt_template::composition()
        .declare()
        .and_then(Declared::admit_unprofiled)
        .expect("the template composition admits")
}

#[must_use]
pub fn member<'a>(admitted: &'a Admitted, id: &str) -> &'a AdmittedMember {
    admitted
        .built()
        .members()
        .find(|member| member.owner.as_str() == id)
        .unwrap_or_else(|| panic!("the composition admits the `{id}` member"))
}

#[must_use]
pub fn member_jobs(id: &str) -> Vec<Arc<dyn Job>> {
    let admitted = admitted();
    member(&admitted, id)
        .sink
        .jobs()
        .iter()
        .map(|job| Arc::clone(job.job()))
        .collect()
}
