//! One conversation as a document, not a table.
//!
//! The table export (`export::datasets`) answers "what did the ledger show";
//! a reader who wants to know *why* a conversation went the way it did needs
//! the record underneath it — every message body, every tool call with its
//! input and result, every governance decision with the rules it evaluated,
//! every scanner finding and every hook event the harness reported — in one
//! file. That is nested, so it is a [`bundle::ConversationBundle`] serialised
//! as JSON, rendered as Markdown for reading, or streamed one conversation
//! per line as JSON Lines for a whole filtered set.

pub(crate) mod bundle;
pub(crate) mod handler;
pub(crate) mod links;
pub(crate) mod markdown;
pub(crate) mod selection;
pub(crate) mod view;

pub(crate) use links::DocumentExportView;
