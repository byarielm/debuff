pub mod accounts;
pub mod definitions;
pub mod ingest;
pub mod labels;
pub mod moderators;
pub mod queue;
pub mod types;
pub mod webhooks;

use axum::routing::{delete, get, patch, post};
use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/moderators",
            post(moderators::add_moderator).get(moderators::list_moderators),
        )
        .route(
            "/moderators/{did}",
            delete(moderators::remove_moderator),
        )
        .route(
            "/definitions",
            get(definitions::list_definitions).post(definitions::create_definition),
        )
        .route(
            "/definitions/{id}",
            get(definitions::get_definition)
                .patch(definitions::update_definition)
                .delete(definitions::delete_definition),
        )
        .route(
            "/labels",
            get(labels::query_labels)
                .post(labels::apply_labels)
                .delete(labels::negate_labels),
        )
        .route(
            "/queue",
            get(queue::list_queue),
        )
        .route(
            "/queue/{id}",
            get(queue::get_queue_item).patch(queue::update_status),
        )
        .route(
            "/queue/{id}/assign",
            patch(queue::assign_moderator),
        )
        .route(
            "/queue/{id}/escalate",
            post(queue::escalate),
        )
        .route(
            "/queue/{id}/notes",
            post(queue::add_note),
        )
        .route(
            "/accounts/{did}",
            get(accounts::get_account),
        )
        .route(
            "/accounts/{did}/action",
            post(accounts::apply_action),
        )
        .route(
            "/webhooks",
            get(webhooks::list_webhooks).post(webhooks::create_webhook),
        )
        .route(
            "/webhooks/{id}",
            patch(webhooks::update_webhook).delete(webhooks::delete_webhook),
        )
        .route("/ingest", post(ingest::ingest))
}
