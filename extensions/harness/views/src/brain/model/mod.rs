//! The pure state of the Brain's views.
//!
//! It holds which facts are listed and how they
//! are filtered and paged, the open fact with its timeline and why-view, the
//! actions a person may take on it, the proposals inbox, whether the brain can
//! be used at all, and which reply of a session is its latest. Nothing here
//! touches GPUI or the network.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod actions;
mod availability;
mod detail;
mod failure;
mod filter;
mod listing;
mod load;
mod proposals;
mod replies;

pub use actions::{ActionOutcome, Decision, DecisionOutcome, FactAction};
pub use availability::Availability;
pub use detail::FactDetail;
pub use failure::BrainFailure;
pub use filter::{FactFilter, FilterError, PAGE_SIZE};
pub use listing::FactListing;
pub use load::Load;
pub use proposals::ProposalInbox;
pub use replies::LatestReplies;
