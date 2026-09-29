//! One account in a server-rendered picker: the gateway probe and the
//! access-control "Check a person" tab both offer the same list.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PickableUserView {
    pub id: String,
    pub label: String,
    pub selected: bool,
}
