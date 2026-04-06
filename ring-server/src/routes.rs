use axum::routing::{get, post};
use axum::Router;

use crate::handlers::ai;
use crate::handlers::conversation;
use crate::handlers::install;
use crate::handlers::ring;
use crate::handlers::setup;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let setup_routes = Router::new()
        .route("/status", get(setup::get_status))
        .route("/username", post(setup::set_username))
        .route("/llm", post(setup::set_llm))
        .route("/gitlab", post(setup::set_gitlab))
        .route("/complete", post(setup::complete));

    let ring_routes = Router::new()
        .route("/", get(ring::list_rings).post(ring::create_ring))
        .route(
            "/{ringId}",
            get(ring::get_ring)
                .put(ring::update_ring)
                .delete(ring::delete_ring),
        );

    let conversation_routes = Router::new()
        .route("/", get(conversation::list).post(conversation::create))
        .route("/{convId}", get(conversation::get))
        .route(
            "/{convId}/messages",
            get(conversation::get_messages).post(conversation::send_message),
        );

    Router::new()
        .nest("/api/v1/setup", setup_routes)
        .nest("/api/v1/rings", ring_routes)
        .nest("/api/v1/rings/{ringId}/conversations", conversation_routes)
        .route("/api/v1/super-ring/chat", post(ai::super_ring_chat))
        .route("/join", get(install::join_page))
        .with_state(state)
}
