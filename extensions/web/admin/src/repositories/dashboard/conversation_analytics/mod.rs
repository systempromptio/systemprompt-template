//! Links between a session and the entities it exercised.

mod entity_links;

pub use entity_links::{EntityLinkInput, list_session_entity_links, upsert_session_entity_link};
