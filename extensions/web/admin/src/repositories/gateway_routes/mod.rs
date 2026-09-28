//! The `gateway_routes` table: `services/ai/gateway.yaml`'s `routes:`
//! sequence as the console holds it.
//!
//! Core boots its dispatcher from the file and never reads this table. The
//! table is the console's ledger of the same routes: the boot job seeds it
//! from the file when it is empty, and every sync apply ends by regenerating
//! the file's `routes:` sequence from the rows ([`render`]) so the next
//! restart dispatches what the table says. The sync plane in
//! `sync::gateway_routes` compares the two and offers the three directions.

pub mod declared;
pub mod drift;
pub mod render;
pub mod rows;
