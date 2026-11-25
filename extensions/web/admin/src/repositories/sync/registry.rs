//! The planes this instance syncs, in page order.
//!
//! One line per plane; the page renders one card per entry here, so adding
//! a plane is an `impl` and a line. `access_control` first because it is
//! the entitlement, `groups` next because the rules name its ids, and
//! `gateway_policies` after it because the console also edits it in place,
//! then `governance`, whose runtime input is a file core reads at boot. The
//! gateway routes are not a plane: they are core's catalog
//! (`services/ai/gateway.yaml` in the catalog shape), which the legacy
//! route projection cannot read.

use super::access_control::AccessControlPlane;
use super::gateway_policies::GatewayPoliciesPlane;
use super::governance::GovernancePlane;
use super::groups::GroupsPlane;
use super::plane::SyncPlane;

#[must_use]
pub fn planes() -> Vec<Box<dyn SyncPlane>> {
    vec![
        Box::new(AccessControlPlane),
        Box::new(GroupsPlane),
        Box::new(GatewayPoliciesPlane),
        Box::new(GovernancePlane),
    ]
}

#[must_use]
pub fn find_plane(id: &str) -> Option<Box<dyn SyncPlane>> {
    planes().into_iter().find(|p| p.id() == id)
}
