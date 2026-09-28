//! The planes this instance syncs, in page order.
//!
//! One line per plane; the page renders one card per entry here, so adding
//! a plane is an `impl` and a line. `access_control` first because it is
//! the entitlement, `groups` next because the rules name its ids, and
//! `gateway_policies` after it because the console also edits it in place,
//! then the two planes whose runtime input is a file core reads at boot:
//! `gateway_routes` and `governance`.

use super::access_control::AccessControlPlane;
use super::gateway_policies::GatewayPoliciesPlane;
use super::gateway_routes::GatewayRoutesPlane;
use super::governance::GovernancePlane;
use super::groups::GroupsPlane;
use super::plane::SyncPlane;

#[must_use]
pub fn planes() -> Vec<Box<dyn SyncPlane>> {
    vec![
        Box::new(AccessControlPlane),
        Box::new(GroupsPlane),
        Box::new(GatewayPoliciesPlane),
        Box::new(GatewayRoutesPlane),
        Box::new(GovernancePlane),
    ]
}

#[must_use]
pub fn find_plane(id: &str) -> Option<Box<dyn SyncPlane>> {
    planes().into_iter().find(|p| p.id() == id)
}
