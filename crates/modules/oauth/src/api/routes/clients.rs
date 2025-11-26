use crate::api::rest;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<systemprompt_core_system::AppContext> {
    Router::new()
        .route("/", get(rest::client::list::list_clients))
        .route("/", post(rest::client::create::create_client))
        .route("/{client_id}", get(rest::client::get::get_client))
        .route("/{client_id}", put(rest::client::update::update_client))
        .route("/{client_id}", delete(rest::client::delete::delete_client))
}
